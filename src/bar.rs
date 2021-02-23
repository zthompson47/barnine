use std::io::Write;

use swaybar_types::{Align, Block, MinWidth};
use tokio::sync::mpsc;

use crate::err::Res;

pub static MAX_WINDOW_NAME_LENGTH: usize = 80;

#[derive(Debug)]
pub enum Update {
    BatteryCapacity(Option<String>),
    BatteryStatus(Option<String>),
    Brightness(Option<u32>),
    Redraw,
    Time(Option<String>),
    Volume(Option<u32>),
    Mute(Option<bool>),
    WindowName(Option<String>),
}

#[derive(Debug, Default)]
pub struct Bar {
    pub battery_status: Option<String>,
    pub battery_capacity: Option<String>,
    pub brightness: Option<u32>,
    pub window_name: Option<String>,
    pub time: Option<String>,
    pub volume: Option<u32>,
    pub mute: Option<bool>,
}

impl Bar {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn write_json(
        &mut self,
        writer: &mut dyn Write,
        mut rx_updates: mpsc::UnboundedReceiver<Update>,
    ) {
        while let Some(cmd) = rx_updates.recv().await {
            match cmd {
                Update::Redraw => {
                    write!(writer, "{},\n", self.to_json().unwrap()).unwrap();
                    writer.flush().unwrap();
                }
                Update::BatteryCapacity(x) => self.battery_capacity = x,
                Update::BatteryStatus(x) => self.battery_status = x,
                Update::Brightness(x) => self.brightness = x,
                Update::Time(x) => self.time = x,
                Update::Mute(x) => self.mute = x,
                Update::Volume(x) => self.volume = x,
                Update::WindowName(x) => self.window_name = x,
            }
        }
    }

    pub fn to_json(&self) -> Res<String> {
        let mut result = Vec::<String>::new();

        if self.brightness.is_some() {
            let block = Block {
                full_text: String::from(format!("{:2.0}", self.brightness.unwrap())),
                background: Some("#004400".to_string()),
                min_width: Some(MinWidth::Pixels(200)),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block).unwrap());
        }

        if self.battery_status.is_some() {
            let block_status = Block {
                full_text: self.battery_status.as_ref().unwrap().to_string(),
                background: Some("#880000".to_string()),
                separator_block_width: Some(0),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block_status).unwrap());
        }

        if self.battery_capacity.is_some() {
            let block_capacity = Block {
                full_text: self.battery_capacity.as_ref().unwrap().to_string(),
                background: Some("#990000".to_string()),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block_capacity).unwrap());
        }

        if self.window_name.is_some() {
            let window_name = String::from(self.window_name.as_ref().unwrap());
            let short_window_name = truncate(&window_name, MAX_WINDOW_NAME_LENGTH);
            let short_window_name = format!("{}...", short_window_name);
            let block = Block {
                align: Some(Align::Left),
                full_text: window_name,
                short_text: Some(short_window_name),
                background: Some("#000000".to_string()),
                // min_width: Some(MinWidth::Percent(100)),
                min_width: Some(MinWidth::Pixels(1300)),
                ..Block::default()
            };

            result.push(serde_json::to_string(&block).unwrap());
        }

        if self.mute.is_some() {
            let block = Block {
                align: Some(Align::Center),
                full_text: self.mute.as_ref().unwrap().to_string(),
                background: Some("#000000".to_string()),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block).unwrap());
        }

        if self.volume.is_some() {
            let block = Block {
                align: Some(Align::Center),
                full_text: self.volume.as_ref().unwrap().to_string(),
                background: Some("#000000".to_string()),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block).unwrap());
        }

        if self.time.is_some() {
            let block = Block {
                align: Some(Align::Right),
                full_text: String::from(self.time.as_ref().unwrap()),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block).unwrap());
        }

        Ok(format!("[{}]", result.join(",")))
    }
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Result, Value};
    use swaybar_types::Block;
    use tokio::sync::mpsc;

    use super::{Bar, Update};

    #[test]
    fn json_output_empty() {
        let d = Bar::default();
        let j = d.to_json().unwrap();

        assert_eq!("[]", j.as_str());
    }

    #[test]
    fn json_output_full() -> Result<()> {
        let d = Bar {
            battery_status: Some("Full".into()),
            battery_capacity: Some("99".into()),
            brightness: Some(1_000),
            window_name: Some("Window".into()),
            time: Some("12:01".into()),
            volume: Some(22_000),
            mute: Some(false),
        };
        let j = d.to_json().unwrap();
        assert!(j.len() > 2);

        // Basic sanity test: valid json and an expected value
        let json: Value = serde_json::from_str(&j)?;
        let first = &json[0];
        assert_eq!("1000", &first["full_text"]);

        Ok(())
    }

    #[tokio::test]
    async fn json_from_updates() {
        let mut bar = Bar::new();
        let (tx_updates, rx_updates) = mpsc::unbounded_channel::<Update>();
        tx_updates.send(Update::Time(Some("12:01".into()))).unwrap();
        tx_updates
            .send(Update::BatteryStatus(Some("Full".into())))
            .unwrap();
        tx_updates
            .send(Update::BatteryCapacity(Some("88".into())))
            .unwrap();
        tx_updates.send(Update::Redraw).unwrap();
        drop(tx_updates);

        let mut json = Vec::new();
        bar.write_json(&mut json, rx_updates).await;
        let json = String::from_utf8(json).unwrap();

        let output: Vec<Block> = serde_json::from_str(&json).unwrap();

        println!("{:?}", output);
        assert_eq!(3, output.len());
        assert_eq!("Full", output[0].full_text);
        assert_eq!("88", output[1].full_text);
        assert_eq!("12:01", output[2].full_text);
    }
}
