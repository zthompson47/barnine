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
    WindowName(Option<String>),
}

#[derive(Default)]
pub struct Display {
    pub battery_status: Option<String>,
    pub battery_capacity: Option<String>,
    pub brightness: Option<u32>,
    pub window_name: Option<String>,
    pub time: Option<String>,
    pub volume: Option<u32>,
    tx_redraw: Option<mpsc::UnboundedSender<String>>,
}

impl Display {
    pub fn new(tx: mpsc::UnboundedSender<String>) -> Self {
        Display {
            tx_redraw: Some(tx),
            ..Display::default()
        }
    }

    pub fn update(&mut self, command: Update) {
        match command {
            Update::BatteryCapacity(val) => self.battery_capacity = val,
            Update::BatteryStatus(val) => self.battery_status = val,
            Update::Brightness(val) => self.brightness = val,
            Update::Redraw => self.redraw(),
            Update::Time(val) => self.time = val,
            Update::Volume(val) => self.volume = val,
            Update::WindowName(val) => self.window_name = val,
        };
    }

    fn redraw(&self) {
        if self.tx_redraw.is_some() {
            self.tx_redraw
                .as_ref()
                .unwrap()
                .send(self.to_json().unwrap())
                .unwrap();
        }
    }

    pub fn to_json(&self) -> Res<String> {
        let mut result = Vec::<String>::new();

        if self.brightness.is_some() {
            let block = Block {
                full_text: String::from(format!("{:2.0}", self.brightness.as_ref().unwrap())),
                background: Some("#004400".to_string()),
                min_width: Some(MinWidth::Pixels(200)),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block).unwrap());
        }

        if self.battery_status.is_some() {
            let block = Block {
                full_text: String::from(self.battery_status.as_ref().unwrap()),
                background: Some("#880000".to_string()),
                separator_block_width: Some(0),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block).unwrap());
        }

        if self.battery_capacity.is_some() {
            let block = Block {
                full_text: String::from(self.battery_capacity.as_ref().unwrap()),
                background: Some("#990000".to_string()),
                ..Block::default()
            };
            result.push(serde_json::to_string(&block).unwrap());
        }

        if self.window_name.is_some() {
            let window_name = String::from(self.window_name.as_ref().unwrap());
            // let mut short_window_name = window_name.clone();
            // short_window_name.truncate(MAX_WINDOW_NAME_LENGTH);
            let short_window_name = truncate(&window_name, MAX_WINDOW_NAME_LENGTH);

            let short_window_name = format!("{}...", short_window_name);
            let block = Block {
                align: Some(Align::Center),
                full_text: window_name,
                short_text: Some(short_window_name),
                background: Some("#000000".to_string()),
                // min_width: Some(MinWidth::Percent(100)),
                min_width: Some(MinWidth::Pixels(1300)),
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

#[cfg(test)]
mod tests {
    use serde_json::{Result, Value};
    use swaybar_types::Block;
    use tokio::sync::mpsc::unbounded_channel;

    use super::{Display, Update};

    #[test]
    fn json_output_empty() {
        let d = Display::default();
        let j = d.to_json().unwrap();

        assert_eq!("[]", j.as_str());
    }

    #[test]
    fn json_output_full() -> Result<()> {
        let d = Display {
            battery_status: Some("Full".into()),
            battery_capacity: Some("99".into()),
            brightness: Some(1_000),
            window_name: Some("Window".into()),
            time: Some("12:01".into()),
            volume: Some(22_000),
            tx_redraw: None,
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
        let (tx_dr, mut rx_dr) = unbounded_channel::<String>();
        let mut d = Display::new(tx_dr);

        let (tx_up, mut rx_up) = unbounded_channel::<Update>();
        tokio::spawn(async move {
            while let Some(command) = rx_up.recv().await {
                d.update(command);
            }
        });
        tx_up.send(Update::Time(Some("12:01".into()))).unwrap();
        tx_up
            .send(Update::BatteryStatus(Some("Full".into())))
            .unwrap();
        tx_up
            .send(Update::BatteryCapacity(Some("88".into())))
            .unwrap();
        tx_up.send(Update::Redraw).unwrap();

        let mut got_there = false;
        while let Some(json) = rx_dr.recv().await {
            got_there = true;
            let output: Vec<Block> = serde_json::from_str(&json).unwrap();
            assert_eq!(3, output.len());
            assert_eq!("Full", output[0].full_text);
            assert_eq!("88", output[1].full_text);
            assert_eq!("12:01", output[2].full_text);
            break;
        }
        assert!(got_there);
    }
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}
