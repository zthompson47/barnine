use swayipc_async::{Connection, Event, EventType, Node, WindowChange, WindowEvent};

use log::debug;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

use crate::bar::Update;
use crate::err::Res;

pub async fn get_sway(tx: UnboundedSender<Update>) -> Res<()> {
    let subs = [EventType::Window];
    let mut events = Connection::new().await?.subscribe(&subs).await?;

    while let Some(event) = events.next().await {
        match event? {
            Event::Window(window_event) => match *window_event {
                WindowEvent {
                    change: WindowChange::Focus,
                    container:
                        Node {
                            name: window_name, ..
                        },
                    ..
                }
                | WindowEvent {
                    change: WindowChange::Title,
                    container:
                        Node {
                            name: window_name, ..
                        },
                    ..
                } => {
                    if !window_name.is_some() {
                        debug!("Window change with None window_name");
                    }
                    debug!("CHGNE WINDO title:{:?}", window_name);
                    tx.send(Update::WindowName(Some(window_name.unwrap())))?;
                    tx.send(Update::Redraw)?;
                }

                WindowEvent {
                    change: WindowChange::FullscreenMode,
                    ..
                } => {}

                _ => {
                    debug!("-in->>{:?}", window_event);
                }
            },
            _ => {
                debug!("-out->>");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::bar::{Display, MAX_WINDOW_NAME_LENGTH};

    #[test]
    #[should_panic]
    fn truncate_on_char_boundary() {
        let utf8_3_bytes = "ท";
        let mut s = String::from(utf8_3_bytes.repeat(2));
        s.truncate(1);
    }

    #[test]
    fn window_name_with_char_boundary() {
        let utf8_3_bytes = "ท";
        let test_str = utf8_3_bytes.repeat(MAX_WINDOW_NAME_LENGTH + 1);
        let mut display = Display::default();
        display.window_name = Some(String::from(test_str));

        // Force truncation on a char boundary
        let json = display.to_json().unwrap();

        assert!(json.len() > 0);
    }
}
