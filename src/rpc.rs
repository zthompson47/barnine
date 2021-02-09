use std::env::var;
use std::fs::remove_file;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use log::debug;
use smol::channel::Sender;
use smol::net::unix::{UnixListener, UnixStream};
use smol::prelude::*;
use smol::spawn;

use crate::{
    bar::Update,
    brightness::brighten,
    brightness::Brightness::{Keyboard, Screen},
    brightness::Delta::{DownPct, UpPct},
    err::{Error, Res},
    volume::{volume, Volume},
};

pub fn socket_path() -> PathBuf {
    //! Initialize unix socket in system runtime dir
    let runtime_dir = var("XDG_RUNTIME_DIR").unwrap_or(String::from("/tmp"));
    Path::new(&runtime_dir).join("barnine.sock")
}

pub async fn get_rpc(tx: Sender<Update>) -> Res<()> {
    // Open unix socket
    let sock = socket_path();
    let _ = remove_file(&sock);
    let listener = UnixListener::bind(&sock)?;
    let mut incoming = listener.incoming();

    // Serve clients
    while let Some(stream) = incoming.next().await {
        match stream {
            Ok(stream) => {
                spawn(handle_connection(stream, tx.clone())).detach();
            }
            Err(err) => {
                debug!("got err: {:?}", err);
                return Err(Error::from(err));
            }
        }
    }

    async fn handle_connection(mut stream: UnixStream, tx: Sender<Update>) -> Res<()> {
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
                    tx.send(Update::Brightness(Some(new_val))).await?;
                    tx.send(Update::Redraw).await?;
                }

                let volume_delta = match msg {
                    "volume_up" => Some(Volume::Speakers(UpPct(2))),
                    "volume_down" => Some(Volume::Speakers(DownPct(2))),
                    _ => None,
                };
                if volume_delta.is_some() {
                    match volume(volume_delta.unwrap()).await {
                        Ok(new_vol) => {
                            tx.send(Update::Volume(Some(new_vol))).await?;
                            tx.send(Update::Redraw).await?;
                        },
                        Err(err) => debug!("{:?}", err),
                    }
                }
            }
        }
        Ok(())
    }

    Ok(())
}
