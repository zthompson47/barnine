use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::spawn;
use tokio::sync::mpsc;
use tracing::{debug, trace};

use crate::{
    bar::Update,
    brightness::brighten,
    brightness::Brightness::{Keyboard, Screen},
    brightness::Delta::{DownPct, UpPct},
    err::Res,
    nine::NineCmd,
    pulse::{get_mute, toggle_mute},
    volume::{volume, Volume},
};

pub fn get_socket_path(app_name: &str) -> PathBuf {
    //! Initialize unix socket in system runtime dir

    // Look for APPNAME_DEV_DIR environment variable to override default
    let mut dev_dir = app_name.to_uppercase();
    dev_dir.push_str("_DEV_DIR");

    let mut file_name = app_name.to_string();
    file_name.push_str(".sock");

    match env::var(dev_dir) {
        Ok(dev_dir) => Path::new(&dev_dir).join(file_name),
        Err(_) => {
            let run_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
            Path::new(&run_dir).join(app_name).join(file_name)
        }
    }
}

pub async fn watch_rpc(tx: mpsc::UnboundedSender<Update>) -> Res<()> {
    trace!("Starting get_rpc");
    let sock = get_socket_path("barnine");
    let _ = fs::remove_file(&sock);
    if let Some(base_dir) = sock.parent() {
        fs::create_dir_all(base_dir)?;
    }
    let listener = UnixListener::bind(&sock)?;

    loop {
        if let Ok((stream, _addr)) = listener.accept().await {
            spawn(handle_connection(stream, tx.clone()));
        }
    }
}

async fn handle_connection(mut stream: UnixStream, tx: mpsc::UnboundedSender<Update>) -> Res<()> {
    let mut buf = vec![0u8; 64];
    if let Ok(len) = stream.read(&mut buf).await {
        if let Ok(msg) = from_utf8(&buf[0..len]) {
            let brightness_delta = match msg {
                "brightness_up" => Some(Screen(UpPct(5))),
                "brightness_down" => Some(Screen(DownPct(5))),
                "kbd_up" => Some(Keyboard(UpPct(5))),
                "kbd_down" => Some(Keyboard(DownPct(5))),
                _ => None,
            };
            if brightness_delta.is_some() {
                let new_val = brighten(brightness_delta.unwrap()).await?;
                tx.send(Update::Brightness(Some(new_val)))?;
                tx.send(Update::Redraw)?;
            }

            let volume_delta = match msg {
                "volume_up" => Some(Volume::Speakers(UpPct(2))),
                "volume_down" => Some(Volume::Speakers(DownPct(2))),
                _ => None,
            };
            if volume_delta.is_some() {
                match volume(volume_delta.unwrap()).await {
                    Ok(new_vol) => {
                        tx.send(Update::Volume(Some(new_vol)))?;
                        tx.send(Update::Redraw)?;
                    }
                    Err(err) => {
                        debug!("{:?}", err);
                    }
                }
            }
            if let "toggle_mute" = msg {
                toggle_mute().await.unwrap();
                tx.send(Update::Mute(Some(get_mute().await.unwrap())))
                    .unwrap();
                tx.send(Update::Redraw)?;
            }

            use NineCmd::*;

            if let "move_left" = msg {
                tx.send(Update::Nine(MoveLeft)).unwrap();
                tx.send(Update::Redraw)?;
            }
            if let "move_right" = msg {
                tx.send(Update::Nine(MoveRight)).unwrap();
                tx.send(Update::Redraw)?;
            }
            if let "move_up" = msg {
                tx.send(Update::Nine(MoveUp)).unwrap();
                tx.send(Update::Redraw)?;
            }
            if let "move_down" = msg {
                tx.send(Update::Nine(MoveDown)).unwrap();
                tx.send(Update::Redraw)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tokio::{
        io::AsyncWriteExt,
        net::UnixStream,
        sync::mpsc,
        time::{sleep, Duration},
    };

    use super::{get_socket_path, watch_rpc};
    use crate::bar::Update;

    #[tokio::test]
    async fn run_rpc_socket() {
        // Construct expected socket path
        let app_name = "barnine";
        let dir = tempdir().unwrap().into_path();
        let mut sock_path = dir.join(app_name).join(app_name);
        sock_path.set_extension("sock");

        // Check against actual socket path
        std::env::remove_var("BARNINE_DEV_DIR");
        std::env::set_var("XDG_RUNTIME_DIR", &dir);
        assert_eq!(sock_path, get_socket_path(app_name));

        // Start rpc listener task
        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::spawn(watch_rpc(tx));

        // Yield for rpc task to start and create socket
        // TODO test fails intermittently..  needs sync on socket file creation
        sleep(Duration::from_secs(0)).await;
        sleep(Duration::from_secs(0)).await;
        assert!(sock_path.exists());

        // Send a command
        let mut conn = UnixStream::connect(sock_path).await.unwrap();
        conn.write_all("volume_down".as_bytes()).await.unwrap();

        // Read the command
        let mut got_it = false;
        if let Some(command) = rx.recv().await {
            match command {
                Update::Volume(Some(_)) => got_it = true,
                _ => {}
            }
        }
        assert!(got_it);
    }

    #[test]
    fn fallback_socket_in_tmp() {
        std::env::remove_var("XDG_RUNTIME_DIR");
        assert!(get_socket_path("foo").starts_with("/tmp"));
    }
}
