use swaybar_types::{Align, Block, MinWidth};
use tokio::sync::mpsc::UnboundedReceiver;

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
    pub rx: Option<UnboundedReceiver<Update>>,
}

impl Display {
    pub async fn run(&mut self) {
        while let Some(section) = self.rx.as_mut().unwrap().recv().await {
            match section {
                Update::BatteryStatus(val) => self.battery_status = val,
                Update::BatteryCapacity(val) => self.battery_capacity = val,
                Update::Brightness(val) => self.brightness = val,
                Update::WindowName(val) => self.window_name = val,
                Update::Time(val) => self.time = val,
                Update::Volume(val) => self.volume = val,
                Update::Redraw => self.redraw(),
            };
        }
    }

    fn redraw(&self) {
        print!("[");

        if self.brightness.is_some() {
            let block = Block {
                full_text: String::from(format!("{:2.0}", self.brightness.as_ref().unwrap())),
                background: Some("#004400".to_string()),
                min_width: Some(MinWidth::Pixels(200)),
                ..Block::default()
            };
            print!("{},", serde_json::to_string(&block).unwrap());
        }

        if self.battery_status.is_some() {
            let block = Block {
                full_text: String::from(self.battery_status.as_ref().unwrap()),
                background: Some("#880000".to_string()),
                separator_block_width: Some(0),
                ..Block::default()
            };
            print!("{},", serde_json::to_string(&block).unwrap());
        }

        if self.battery_capacity.is_some() {
            let block = Block {
                full_text: String::from(self.battery_capacity.as_ref().unwrap()),
                background: Some("#990000".to_string()),
                ..Block::default()
            };
            print!("{},", serde_json::to_string(&block).unwrap());
        }

        if self.window_name.is_some() {
            let window_name = String::from(self.window_name.as_ref().unwrap());
            let mut short_window_name = window_name.clone();
            short_window_name.truncate(80);
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
            print!("{},", serde_json::to_string(&block).unwrap());
        }



        if self.volume.is_some() {
            let block = Block {
                align: Some(Align::Center),
                full_text: self.volume.as_ref().unwrap().to_string(),
                background: Some("#000000".to_string()),
                ..Block::default()
            };
            print!("{},", serde_json::to_string(&block).unwrap());
        }




        if self.time.is_some() {
            let block = Block {
                align: Some(Align::Right),
                full_text: String::from(self.time.as_ref().unwrap()),
                ..Block::default()
            };
            print!("{},", serde_json::to_string(&block).unwrap());
        }

        println!("],");
    }
}
