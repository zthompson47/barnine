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

static BATTERY_UEVENT: &str = "/sys/class/power_supply/BAT0/uevent";

fn main() {
    smol::block_on(async_log("log.txt", main_loop()));
}

async fn main_loop() {
    debug!("started main loop");

    let header = Header {
        version: Version::One,
        stop_signal: None,
        cont_signal: None,
        click_events: None,
    };
    println!("{}", serde_json::to_string(&header).unwrap());
    println!("[");

    let (tx, rx) = channel::unbounded();

    smol::spawn(process_sway_events(tx.clone())).detach();
    smol::spawn(get_batt(tx.clone())).detach();
    smol::spawn(get_time(tx.clone())).detach();

    Display {
        battery_status: None,
        battery_capacity: None,
        window_name: None,
        time: None,
        rx,
    }.run().await;

    unreachable!();
}

#[derive(Debug)]
enum Section {
    #[allow(dead_code)]
    BatteryStatus(String),
    #[allow(dead_code)]
    BatteryCapacity(String),
    WindowName(String),
    #[allow(dead_code)]
    Time(String),
}

struct Display {
    battery_status: Option<String>,
    battery_capacity: Option<String>,
    window_name: Option<String>,
    time: Option<String>,
    rx: channel::Receiver<Section>,
}

impl Display {
    async fn run(&mut self) {
        while let Ok(section) = self.rx.recv().await {
            match section {
                Section::BatteryStatus(val) => self.battery_status = Some(val),
                Section::BatteryCapacity(val) => self.battery_capacity = Some(val),
                Section::WindowName(val) => self.window_name = Some(val),
                Section::Time(val) => self.time = Some(val),
            };
            self.redraw();
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

async fn get_time(tx: channel::Sender<Section>) {
    loop {
        let now: DateTime<Local> = Local::now();
        let fmt_now = now.format("%b %d %A %l:%M:%S %p").to_string();
        tx.send(Section::Time(fmt_now))
            .await
            .unwrap();
        sleep(Duration::from_secs(1)).await;
    }
}

async fn get_batt(tx: channel::Sender<Section>) -> io::Result<()> {
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

        tx.send(Section::BatteryStatus(bs))
            .await
            .unwrap();

        tx.send(Section::BatteryCapacity(bc))
            .await
            .unwrap();

        sleep(Duration::from_secs(5)).await;
    }
}

async fn process_sway_events(tx: channel::Sender<Section>) -> Fallible<()> {
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
                    } => {
                        tx.send(Section::WindowName(window_name.unwrap()))
                            .await
                            .unwrap();
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
    Ok(())
}
