use swayipc_async::{
    Connection,
    Event,
    EventType,
    Fallible,
    Node,
    WindowChange,
    WindowEvent,
};

use log::debug;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

use crate::bar::Update;

pub async fn get_sway(tx: UnboundedSender<Update>) -> Fallible<()> {
    let subs = [
        EventType::Window,
    ];
    let mut events = Connection::new().await?.subscribe(&subs).await?;

    while let Some(event) = events.next().await {
        match event? {
            Event::Window(window_event) => {
                match *window_event {
                    WindowEvent {
                        change: WindowChange::Focus,
                        container: Node {
                            name: window_name,
                            ..
                        },
                        ..
                    }
                    | WindowEvent {
                        change: WindowChange::Title,
                        container: Node {
                            name: window_name,
                            ..
                        },
                        ..
                    } => {
                        tx.send(Update::WindowName(Some(window_name.unwrap()))).unwrap();
                        tx.send(Update::Redraw).unwrap();
                    },

                    WindowEvent {
                        change: WindowChange::FullscreenMode,
                        ..
                    } => {},

                    _ => {
                        debug!("-in->>{:?}", window_event);
                    }
                }
            },
            _ => {
                debug!("-out->>");
            }
        }
    }
    Ok(())
}
