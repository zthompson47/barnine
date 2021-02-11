use std::env::var;
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use log::debug;
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::spawn;
use tokio::sync::mpsc;

use crate::{
    bar::Update,
    brightness::brighten,
    brightness::Brightness::{Keyboard, Screen},
    brightness::Delta::{DownPct, UpPct},
    err,
    volume::{volume, Volume},
};

pub fn socket_path() -> PathBuf {
    //! Initialize unix socket in system runtime dir
    let runtime_dir = var("XDG_RUNTIME_DIR").unwrap_or(String::from("/tmp"));
    Path::new(&runtime_dir).join("barnine.sock")
}

pub async fn get_rpc(tx: mpsc::UnboundedSender<Update>) -> err::Res<()> {
    let sock = socket_path();
    let _ = remove_file(&sock);
    let listener = UnixListener::bind(&sock)?;

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                spawn(handle_connection(stream, tx.clone()));
            }
            Err(_) => {}
        }
    }

}

async fn handle_connection(
    mut stream: UnixStream,
    tx: mpsc::UnboundedSender<Update>,
) -> err::Res<()> {
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
                    Err(err) => debug!("{:?}", err),
                }
            }
        }
    }
    Ok(())
}
