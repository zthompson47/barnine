use allotropic::async_log;
use async_std::task::sleep;
use chrono::prelude::*;
use log::debug;
use smol::{self, channel, stream::StreamExt};
use std::{
    collections::HashMap,
    fs,
    io::{self, prelude::*, BufReader},
    time::Duration,
};
use swaybar_types::{Align, Block, Header, Version};
use swayipc_async::{
    Connection,
    Event,
    EventType,
    Fallible,
    Node,
    WindowChange,
    WindowEvent,
};

const BATTERY_UEVENT: &str = "/sys/class/power_supply/BAT0/uevent";
const BRIGHTNESS_MAX: &str = "/sys/class/backlight/intel_backlight/max_brightness";
const BRIGHTNESS: &str = "/sys/class/backlight/intel_backlight/brightness";
const LOG_FILE: &str = "log.txt";

fn main() {
    smol::block_on(async_log(LOG_FILE, run()));
}

async fn run() {
    debug!("start main loop");

    // Send header
    let header = Header {
        version: Version::One,
        stop_signal: None,
        cont_signal: None,
        click_events: None,
    };
    println!("{}", serde_json::to_string(&header).unwrap());

    // Begin infinite array of updates
    println!("[");

    // Spawn workers
    let (tx, rx) = channel::unbounded();
    smol::spawn(get_battery(tx.clone())).detach();
    smol::spawn(get_brightness(tx.clone())).detach();
    smol::spawn(get_sway(tx.clone())).detach();
    smol::spawn(get_time(tx.clone())).detach();

    // Update display
    Display {
        rx: Some(rx),
        ..Display::default()
    }.run().await;

    unreachable!();
}

#[derive(Debug)]
enum Update {
    BatteryStatus(String),
    BatteryCapacity(String),
    Brightness(f32),
    WindowName(String),
    Time(String),
    Redraw,
}

#[derive(Default)]
struct Display {
    battery_status: Option<String>,
    battery_capacity: Option<String>,
    brightness: Option<f32>,
    window_name: Option<String>,
    time: Option<String>,
    rx: Option<channel::Receiver<Update>>,
}

impl Display {
    async fn run(&mut self) {
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

async fn _get_volume(_tx: channel::Sender<Update>) {
    loop {
    }
}

async fn get_brightness(tx: channel::Sender<Update>) -> io::Result<()> {
    let brt_max = fs::read_to_string(BRIGHTNESS_MAX)?;
    let brt_max = brt_max.trim();
    debug!("brt_max: {:?}", brt_max);
    loop {
        let brt = fs::read_to_string(BRIGHTNESS)?;
        let brt = brt.trim();
        debug!("brt: {:?}", brt);
        let b = brt.parse::<f32>().unwrap();
        let m = brt_max.parse::<f32>().unwrap();
        debug!("b/m: {:?}/{:?}", b, m);
        let percent = (b / m) * 100f32;
        debug!("percent: {:?}", percent);
        tx.send(Update::Brightness(percent)).await.unwrap();
        sleep(Duration::from_secs(5)).await;
    }
}

async fn _get_network(_tx: channel::Sender<Update>) {
    loop {
    }
}

async fn get_time(tx: channel::Sender<Update>) {
    loop {
        let now: DateTime<Local> = Local::now();
        let fmt_now = now.format("%b %d %A %l:%M:%S %p").to_string();
        tx.send(Update::Time(fmt_now))
            .await
            .unwrap();
        tx.send(Update::Redraw).await.unwrap();
        sleep(Duration::from_secs(1)).await;
    }
}

async fn get_battery(tx: channel::Sender<Update>) -> io::Result<()> {
    loop {
        let file = fs::File::open(BATTERY_UEVENT)?;
        let reader = BufReader::new(file);

        let mut data: HashMap<String, String> = HashMap::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                let v: Vec<&str> = line.split('=').collect();
                data.insert(v[0].to_string(), v[1].to_string());
            }
        }
        
        let bs = data.get("POWER_SUPPLY_STATUS").unwrap().to_string();
        let bc = data.get("POWER_SUPPLY_CAPACITY").unwrap().to_string();

        tx.send(Update::BatteryStatus(bs))
            .await
            .unwrap();

        tx.send(Update::BatteryCapacity(bc))
            .await
            .unwrap();

        tx.send(Update::Redraw).await.unwrap();

        sleep(Duration::from_secs(5)).await;
    }
}

async fn get_sway(tx: channel::Sender<Update>) -> Fallible<()> {
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
                        tx.send(Update::WindowName(window_name.unwrap()))
                            .await
                            .unwrap();
                        tx.send(Update::Redraw)
                            .await
                            .unwrap();
                    },

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
