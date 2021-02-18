use swayipc_async::{
    Connection, Event, EventType, Node, WindowChange, WindowEvent, WorkspaceChange, WorkspaceEvent,
};

use log::debug;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

use crate::bar::Update;
use crate::err::Res;
use crate::brightness::{brighten, Brightness::Screen, Delta::{DownPct, UpPct}};

pub async fn get_sway(tx: UnboundedSender<Update>) -> Res<()> {
    let subs = [EventType::Window, EventType::Workspace];
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
                    // Get current window name
                    if !window_name.is_some() {
                        debug!("Window change with None window_name");
                    }
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

            Event::Workspace(workspace_event) => match *workspace_event {
                WorkspaceEvent {
                    change: WorkspaceChange::Focus,
                    current: Some(Node { nodes: ref cur_nodes, .. }),
                    old: Some(Node { nodes: ref old_nodes, .. }),
                    ..
                } => {
                    if contains_firefox(cur_nodes) {
                        let new_val = brighten(Screen(DownPct(22))).await?;
                        tx.send(Update::Brightness(Some(new_val)))?;
                        tx.send(Update::Redraw)?;
                    } else if contains_firefox(old_nodes) {
                        let new_val = brighten(Screen(UpPct(22))).await?;
                        tx.send(Update::Brightness(Some(new_val)))?;
                        tx.send(Update::Redraw)?;
                    }
                }
                _ => {}
            },

            _ => {}
        }
    }
    Ok(())
}

fn contains_firefox(nodes: &[Node]) -> bool {
    for node in nodes {
        match node {
            Node {
                app_id: Some(app_id),
                ..
            } => {
                if app_id.starts_with("firefox") {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::contains_firefox;
    use crate::bar::{Display, MAX_WINDOW_NAME_LENGTH};
    use crate::tests;

    #[test]
    fn identify_firefox_node() {
        let node = tests::mock_firefox_node();
        assert!(contains_firefox(&[node]));
    }

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
