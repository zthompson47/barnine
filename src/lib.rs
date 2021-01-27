use smol::channel::Receiver;
use swaybar_types::{Align, Block};

#[derive(Debug)]
pub enum Update {
    BatteryStatus(String),
    BatteryCapacity(String),
    Brightness(f32),
    WindowName(String),
    Time(String),
    Redraw,
}

#[derive(Default)]
pub struct Display {
    pub battery_status: Option<String>,
    pub battery_capacity: Option<String>,
    pub brightness: Option<f32>,
    pub window_name: Option<String>,
    pub time: Option<String>,
    pub rx: Option<Receiver<Update>>,
}

impl Display {
    pub async fn run(&mut self) {
        while let Ok(section) = self.rx.as_ref().unwrap().recv().await {
            match section {
                Update::BatteryStatus(val) => self.battery_status = Some(val),
                Update::BatteryCapacity(val) => self.battery_capacity = Some(val),
                Update::Brightness(_val) => self.brightness = None, // Some(val),
                Update::WindowName(val) => self.window_name = Some(val),
                Update::Time(val) => self.time = Some(val),
                Update::Redraw => self.redraw(),
            };
        }
    }

    fn redraw(&self) {
        print!("[");

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

        if self.brightness.is_some() {
            let block = Block {
                full_text: String::from(format!("{:2.0}", self.brightness.as_ref().unwrap())),
                background: Some("#004400".to_string()),
                ..Block::default()
            };
            print!("{},", serde_json::to_string(&block).unwrap());
        }

        if self.window_name.is_some() {
            let block = Block {
                align: Some(Align::Center),
                full_text: String::from(self.window_name.as_ref().unwrap()),
                background: Some("#000000".to_string()),
                min_width: Some(1500),
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

pub mod backlight {
    use async_std::task::sleep;
    use log::debug;
    use smol::{fs::read_to_string, channel::Sender};
    use std::{error, io, time::Duration};

    use crate::Update;

    const BRIGHTNESS_MAX: &str = "/sys/class/backlight/intel_backlight/max_brightness";
    const BRIGHTNESS: &str = "/sys/class/backlight/intel_backlight/brightness";

    async fn cur_backlight_with_max(_device: &str) -> (u32, u32) {
        let brt = read_to_string(BRIGHTNESS).await.unwrap();
        let brt_max = read_to_string(BRIGHTNESS_MAX).await.unwrap();

        let brt = brt.trim();
        let brt_max = brt_max.trim();

        let brt = brt.parse::<u32>().unwrap();
        let brt_max = brt_max.parse::<u32>().unwrap();

        (brt, brt_max)
    }

    pub async fn brightness_up() -> Result<(), Box<dyn error::Error>> {
        let (brt, brt_max) = cur_backlight_with_max("intel_backlight").await;
        let increment = brt_max * 5 / 100;
        let new_brt = brt + increment;

        use zbus::azync::connection::Connection;
        let connection = Connection::new_system().await?;
        connection.call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1/session/auto",
            Some("org.freedesktop.login1.Session"),
            "SetBrightness",
            &("backlight", "intel_backlight", new_brt),
        ).await?;
        Ok(())
    }

    pub async fn brightness_down() -> Result<(), Box<dyn error::Error>> {
        let (brt, brt_max) = cur_backlight_with_max("intel_backlight").await;
        let increment = brt_max * 5 / 100;
        let new_brt = brt - increment;

        use zbus::azync::connection::Connection;
        let connection = Connection::new_system().await?;
        connection.call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1/session/auto",
            Some("org.freedesktop.login1.Session"),
            "SetBrightness",
            &("backlight", "intel_backlight", new_brt),
        ).await?;
        Ok(())
    }

    pub async fn _get_brightness(tx: Sender<Update>) -> io::Result<()> {
        let brt_max = smol::fs::read_to_string(BRIGHTNESS_MAX).await?;
        let brt_max = brt_max.trim();
        debug!("brt_max: {:?}", brt_max);
        loop {
            let brt = smol::fs::read_to_string(BRIGHTNESS).await?;
            let brt = brt.trim();
            let b = brt.parse::<f32>().unwrap();
            let m = brt_max.parse::<f32>().unwrap();
            let percent = (b / m) * 100f32;
            tx.send(Update::Brightness(percent)).await.unwrap();
            sleep(Duration::from_secs(5)).await;
        }
    }
}
