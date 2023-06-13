use std::io::Write;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::err::Res;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum StringOrU32 {
    String(String),
    U32(u32),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Block {
    #[serde(skip_serializing)]
    widget: Option<String>,
    #[serde(skip_serializing)]
    char_width: Option<usize>,
    //#[serde(skip_serializing)]
    //format: Option<String>,
    full_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator_block_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_width: Option<StringOrU32>,
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
        if block.background.is_some() && self.background.is_none() {
            self.background = block.background.clone();
        }
        if block.separator_block_width.is_some() && self.separator_block_width.is_none() {
            self.separator_block_width = block.separator_block_width;
        }
        if block.min_width.is_some() && self.min_width.is_none() {
            self.min_width = block.min_width.clone();
        }
        if block.border_top.is_some() && self.border_top.is_none() {
            self.border_top = block.border_top;
        }
        if block.border_bottom.is_some() && self.border_bottom.is_none() {
            self.border_bottom = block.border_bottom;
        }
        if block.border_left.is_some() && self.border_left.is_none() {
            self.border_left = block.border_left;
        }
        if block.border_right.is_some() && self.border_right.is_none() {
            self.border_right = block.border_right;
        }
        if block.align.is_some() && self.align.is_none() {
            self.align = block.align.clone();
        }
        if block.color.is_some() && self.color.is_none() {
            self.color = block.color.clone();
        }
        if block.border.is_some() && self.border.is_none() {
            self.border = block.border.clone();
        }
        if block.separator.is_some() && self.separator.is_none() {
            self.separator = block.separator;
        }
        if block.markup.is_some() && self.markup.is_none() {
            self.markup = block.markup.clone();
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
    Config(Box<Config>),
    Redraw,
    Time(Option<String>),
    Volume(Option<u32>),
    Mute(Option<bool>),
    WindowName(Option<String>),
    Nine(NineCmd),
}

#[derive(Debug)]
pub enum NineCmd {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveTo(i32),
}

use NineCmd::*;

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
    nine: Position,
}

#[derive(Debug, Default)]
enum Position {
    #[default]
    TopLeft,
    MiddleLeft,
    BottomLeft,
    TopMiddle,
    MiddleMiddle,
    BottomMiddle,
    TopRight,
    MiddleRight,
    BottomRight,
}

use Position::*;

impl From<i32> for Position {
    fn from(value: i32) -> Self {
        match value {
            2 => TopLeft,
            1 => MiddleLeft,
            8 => BottomLeft,
            3 => TopMiddle,
            5 => MiddleMiddle,
            4 => BottomMiddle,
            6 => TopRight,
            7 => MiddleRight,
            0 => BottomRight,
            _ => panic!(),
        }
    }
}

impl Position {
    fn num(&self) -> i32 {
        match self {
            TopLeft => 2,
            MiddleLeft => 1,
            BottomLeft => 8,
            TopMiddle => 3,
            MiddleMiddle => 5,
            BottomMiddle => 4,
            TopRight => 6,
            MiddleRight => 7,
            BottomRight => 0,
        }
    }

    fn map_cmd(&self, cmd: NineCmd) -> Self {
        match self {
            TopLeft => match cmd {
                MoveLeft => TopRight,
                MoveRight => TopMiddle,
                MoveUp => BottomLeft,
                MoveDown => MiddleLeft,
                MoveTo(_) => panic!(),
            },
            MiddleLeft => match cmd {
                MoveLeft => MiddleRight,
                MoveRight => MiddleMiddle,
                MoveUp => TopLeft,
                MoveDown => BottomLeft,
                MoveTo(_) => panic!(),
            },
            BottomLeft => match cmd {
                MoveLeft => BottomRight,
                MoveRight => BottomMiddle,
                MoveUp => MiddleLeft,
                MoveDown => TopLeft,
                MoveTo(_) => panic!(),
            },
            TopMiddle => match cmd {
                MoveLeft => TopLeft,
                MoveRight => TopRight,
                MoveUp => BottomMiddle,
                MoveDown => MiddleMiddle,
                MoveTo(_) => panic!(),
            },
            MiddleMiddle => match cmd {
                MoveLeft => MiddleLeft,
                MoveRight => MiddleRight,
                MoveUp => TopMiddle,
                MoveDown => BottomMiddle,
                MoveTo(_) => panic!(),
            },
            BottomMiddle => match cmd {
                MoveLeft => BottomLeft,
                MoveRight => BottomRight,
                MoveUp => MiddleMiddle,
                MoveDown => TopMiddle,
                MoveTo(_) => panic!(),
            },
            TopRight => match cmd {
                MoveLeft => TopMiddle,
                MoveRight => TopLeft,
                MoveUp => BottomRight,
                MoveDown => MiddleRight,
                MoveTo(_) => panic!(),
            },
            MiddleRight => match cmd {
                MoveLeft => MiddleMiddle,
                MoveRight => MiddleLeft,
                MoveUp => TopRight,
                MoveDown => BottomRight,
                MoveTo(_) => panic!(),
            },
            BottomRight => match cmd {
                MoveLeft => BottomMiddle,
                MoveRight => BottomLeft,
                MoveUp => MiddleRight,
                MoveDown => TopRight,
                MoveTo(_) => panic!(),
            },
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Position::*;

        f.write_str(match self {
            TopLeft => "T__",
            MiddleLeft => "M__",
            BottomLeft => "B__",
            TopMiddle => "_T_",
            MiddleMiddle => "_M_",
            BottomMiddle => "_B_",
            TopRight => "__T",
            MiddleRight => "__M",
            BottomRight => "__B",
        })
    }
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
        let mut sway = swayipc_async::Connection::new().await.unwrap();

        while let Some(cmd) = rx_updates.recv().await {
            match cmd {
                Update::Redraw => {
                    writeln!(writer, "{},", self.to_json().unwrap()).unwrap();
                    writer.flush().unwrap();
                }
                Update::BatteryCapacity(val) => self.battery_capacity = val,
                Update::BatteryStatus(val) => self.battery_status = val,
                Update::Brightness(val) => self.brightness = val,
                Update::Config(val) => self.config = *val,
                Update::Time(val) => self.time = val,
                Update::Mute(val) => self.mute = val,
                Update::Volume(val) => self.volume = val,
                Update::WindowName(val) => self.window_name = val,
                Update::Nine(cmd) => {
                    if let MoveTo(num) = cmd {
                        self.nine = Position::from(num);
                        continue;
                    }
                    self.nine = self.nine.map_cmd(cmd);
                    sway.run_command(format!("workspace number {}", self.nine.num()))
                        .await
                        .unwrap();

                    /*
                    let cur = self.nine.num();

                    self.nine = Position::from(match val.unwrap() {
                        10 => {
                            if cur > 2 {
                                cur - 3
                            } else {
                                cur + 6
                            }
                        }

                        11 => {
                            if cur < 6 {
                                cur + 3
                            } else {
                                cur - 6
                            }
                        }
                        12 => {
                            if [0, 3, 6].contains(&cur) {
                                cur + 2
                            } else {
                                cur - 1
                            }
                        }
                        13 => {
                            if [2, 5, 8].contains(&cur) {
                                cur - 2
                            } else {
                                cur + 1
                            }
                        }
                        _ => continue,
                    });
                    */
                }
            }
        }
    }

    pub fn to_json(&mut self) -> Res<String> {
        let mut result = Vec::<String>::new();

        for i in 0..self.config.bar.len() {
            let mut block = self.config.bar[i].borrow_mut();
            match block.widget.as_ref().unwrap().as_str() {
                "time" => {
                    if self.time.is_some() {
                        block.full_text = Some(String::from(self.time.as_ref().unwrap()));
                    }
                }
                "brightness" => {
                    if self.brightness.is_some() {
                        block.full_text = Some(format!("{:>2}{}", self.brightness.unwrap(), "🔅",));
                    }
                }
                "battery" => {
                    if self.battery_capacity.is_some() {
                        block.full_text = Some(format!(
                            "{}{}",
                            self.battery_capacity.as_ref().unwrap(),
                            match self.battery_status {
                                Some(ref val) => match val.as_str() {
                                    "Full" | "Charging" => "🔌",
                                    "Discharging" => "🔋",
                                    _ => "n/a ",
                                },
                                None => "n/a ",
                            },
                        ));
                    }
                }
                "window_name" => {
                    if self.window_name.is_some() {
                        let window_name = String::from(self.window_name.as_ref().unwrap());
                        let max_chars = block.char_width.unwrap_or(100);
                        let short_window_name = truncate(&window_name, max_chars);
                        let short_window_name = format!("{}*", short_window_name);
                        block.full_text = Some(window_name);
                        block.short_text = Some(short_window_name);
                    }
                }
                "volume" => {
                    if let Some(ref v) = self.volume {
                        let pct = v * 100 / 65536;
                        block.full_text = Some(format!(
                            "{:>2}{}",
                            pct,
                            match self.mute {
                                // TODO test missing fields..
                                Some(mute) => match mute {
                                    true => "🔇",
                                    false => "🔈",
                                },
                                None => "🔈",
                            },
                        ));
                    }
                }
                "nine" => {
                    block.full_text = Some(self.nine.to_string());
                }
                _ => {}
            }
            block.load_defaults(&self.config.default.borrow());
            drop(block);
            let block = &self.config.bar[i];
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

    use crate::config::Config;

    //static CONFIG: &str = "[default]\n[[bar]]\nwidget = \"time\"";

    #[test]
    fn json_output_empty() {
        let mut d = Bar::default();
        let j = d.to_json().unwrap();

        assert_eq!("[]", j.as_str());
    }

    #[test]
    fn json_output_full() -> Result<()> {
        let config: Config = toml::from_str(concat!(
            "[default]\n",
            "[[bar]]\n",
            "widget = \"brightness\"\n",
            "[[bar]]\n",
            "widget = \"battery\"\n",
            "[[bar]]\n",
            "widget = \"window_name\"\n",
            "[[bar]]\n",
            "widget = \"volume\"\n",
            "[[bar]]\n",
            "widget = \"time\"\n",
        ))
        .unwrap();
        let mut d = Bar {
            battery_status: Some("Full".into()),
            battery_capacity: Some("99".into()),
            brightness: Some(1_000),
            window_name: Some("Window".into()),
            time: Some("12:01".into()),
            volume: Some(22_000),
            mute: Some(false),
            config,
            ..Default::default()
        };
        let j = d.to_json().unwrap();
        assert!(j.len() > 2);

        // Basic sanity test: valid json and an expected value
        let json: Value = serde_json::from_str(&j)?;
        let first = &json[0];
        assert!(&first["full_text"].as_str().unwrap().starts_with("1000"));

        Ok(())
    }

    #[tokio::test]
    async fn json_from_updates() {
        //let mut bar = Bar::new();
        let config: Config = toml::from_str(concat!(
            "[default]\n",
            "[[bar]]\n",
            "widget = \"battery\"\n",
            "[[bar]]\n",
            "widget = \"time\"\n",
        ))
        .unwrap();
        let mut bar = Bar {
            battery_status: Some("Full".into()),
            battery_capacity: Some("99".into()),
            brightness: None,
            window_name: None,
            time: Some("12:01".into()),
            volume: None,
            mute: None,
            config,
            ..Default::default()
        };

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

        let output: Vec<Block> = serde_json::from_str(json).unwrap();

        assert_eq!(2, output.len());
        assert!(output[0].full_text.starts_with("88"));
        assert_eq!("12:01", output[1].full_text);
    }
}
