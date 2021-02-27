use std::io::Write;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::err::Res;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Block {
    #[serde(skip_serializing)]
    widget: Option<String>,
    #[serde(skip_serializing)]
    width: Option<u32>,
    #[serde(skip_serializing)]
    format: Option<String>,

    full_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator_block_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    align: Option<Align>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_top: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_bottom: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_right: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markup: Option<String>,
}

impl Block {
    fn load_defaults(&mut self, block: &Block) {
        if block.background.is_some() {
            if self.background.is_none() {
                self.background = block.background.clone();
            }
        }
        if block.separator_block_width.is_some() {
            if self.separator_block_width.is_none() {
                self.separator_block_width = block.separator_block_width.clone();
            }
        }
        if block.min_width.is_some() {
            if self.min_width.is_none() {
                self.min_width = block.min_width.clone();
            }
        }
        if block.border_top.is_some() {
            if self.border_top.is_none() {
                self.border_top = block.border_top.clone();
            }
        }
        if block.border_bottom.is_some() {
            if self.border_bottom.is_none() {
                self.border_bottom = block.border_bottom.clone();
            }
        }
        if block.border_left.is_some() {
            if self.border_left.is_none() {
                self.border_left = block.border_left.clone();
            }
        }
        if block.border_right.is_some() {
            if self.border_right.is_none() {
                self.border_right = block.border_right.clone();
            }
        }
        if block.align.is_some() {
            if self.align.is_none() {
                self.align = block.align.clone();
            }
        }
        if block.color.is_some() {
            if self.color.is_none() {
                self.color = block.color.clone();
            }
        }
        if block.border.is_some() {
            if self.border.is_none() {
                self.border = block.border.clone();
            }
        }
        if block.separator.is_some() {
            if self.separator.is_none() {
                self.separator = block.separator.clone();
            }
        }
        if block.markup.is_some() {
            if self.markup.is_none() {
                self.markup = block.markup.clone();
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub enum Update {
    BatteryCapacity(Option<String>),
    BatteryStatus(Option<String>),
    Brightness(Option<u32>),
    Config(Config),
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
    config: Config,
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
                Update::Config(x) => self.config = x,
                Update::Time(x) => self.time = x,
                Update::Mute(x) => self.mute = x,
                Update::Volume(x) => self.volume = x,
                Update::WindowName(x) => self.window_name = x,
            }
        }
    }

    pub fn to_json(&mut self) -> Res<String> {
        //let _background = (&self.config.background).to_string();
        let mut result = Vec::<String>::new();

        //let bar = self.config.bar.borrow_mut();
        for i in 0..self.config.bar.len() {
            //for block in self.config.bar {
            let mut block = self.config.bar[i].borrow_mut();
            match block.widget.as_ref().unwrap().as_str() {
                "time" => if self.time.is_some() {
                    block.full_text = Some(String::from(self.time.as_ref().unwrap()));
                }
                "brightness" => if self.brightness.is_some() {
                    block.full_text =
                        Some(String::from(format!("{:2.0}", self.brightness.unwrap())));
                }
                "battery" => if self.battery_capacity.is_some() {
                    block.full_text = Some(format!(
                        "{} {}%",
                        self.battery_status.as_ref().unwrap().to_string(),
                        self.battery_capacity.as_ref().unwrap().to_string(),
                    ));
                }
                "window_name" => if self.window_name.is_some() {
                    let window_name = String::from(self.window_name.as_ref().unwrap());
                    let short_window_name = truncate(&window_name, 100);
                    let short_window_name = format!("{}...", short_window_name);
                    block.full_text = Some(window_name);
                    block.short_text = Some(short_window_name);
                }
                "volume" => if self.volume.is_some() {
                    let v = self.volume.as_ref().unwrap();
                    let pct = v * 100 / 65536;
                    // let fmt_vol = format!("{:>3}%", pct.to_string());

                    block.full_text = Some(format!(
                        "{} {:>3}%",
                        match self.mute.as_ref().unwrap() {
                            true => "🔇".to_string(),
                            false => "🔈".to_string(),
                        },
                        pct.to_string()
                    ));
                }
                _ => {}
            }
            block.load_defaults(&self.config.default.borrow());
            drop(block);
            let block = &self.config.bar[i];
            result.push(serde_json::to_string(&block).unwrap());
        }

        /*
            if self.brightness.is_some() {
                let block = Block {
                    full_text: Some(String::from(format!("{:2.0}", self.brightness.unwrap()))),
                    ..Block::default()
                };
                result.push(serde_json::to_string(&block).unwrap());
            }

            if self.battery_status.is_some() {
                let block_status = Block {
                    full_text: Some(self.battery_status.as_ref().unwrap().to_string()),
                    ..Block::default()
                };
                result.push(serde_json::to_string(&block_status).unwrap());
            }

            if self.battery_capacity.is_some() {
                let block_capacity = Block {
                    full_text: Some(self.battery_capacity.as_ref().unwrap().to_string()),
                    ..Block::default()
                };
                result.push(serde_json::to_string(&block_capacity).unwrap());
            }

            if self.window_name.is_some() {
                let window_name = String::from(self.window_name.as_ref().unwrap());
                let short_window_name = truncate(&window_name, 100);
                let short_window_name = format!("{}...", short_window_name);
                let block = Block {
                    align: Some(Align::Left),
                    full_text: Some(window_name),
                    short_text: Some(short_window_name),
                    min_width: Some(1300),
                    ..Block::default()
                };

                result.push(serde_json::to_string(&block).unwrap());
            }

            if self.mute.is_some() {
                let block = Block {
                    align: Some(Align::Center),
                    full_text: {
                        match self.mute.as_ref().unwrap() {
                            true => Some("🔇".to_string()),
                            false => Some("🔈".to_string()),
                        }
                    },
                    background: Some(background.clone()),
                    ..Block::default()
                };
                result.push(serde_json::to_string(&block).unwrap());
            }

            if self.volume.is_some() {
                let block = Block {
                    align: Some(Align::Center),
                    full_text: Some({
                        let v = self.volume.as_ref().unwrap();
                        let pct = v * 100 / 65536;
                        format!("{:>3}%", pct.to_string())
                    }),
                    background: Some(background.clone()),
                    ..Block::default()
                };
                result.push(serde_json::to_string(&block).unwrap());
            }

            if self.time.is_some() {
                let block = Block {
                    align: Some(Align::Right),
                    background: Some(background.clone()),
                    full_text: Some(String::from(self.time.as_ref().unwrap())),
                    ..Block::default()
                };
                result.push(serde_json::to_string(&block).unwrap());
            }
        */

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
        // Remove trailing ",\n"
        let json = &json[0..json.len() - 2];

        let output: Vec<Block> = serde_json::from_str(&json).unwrap();

        println!("{:?}", output);
        assert_eq!(3, output.len());
        assert_eq!("Full", output[0].full_text);
        assert_eq!("88", output[1].full_text);
        assert_eq!("12:01", output[2].full_text);
    }
}
