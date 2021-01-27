use std::{
    collections::HashMap,
    fs::{File, remove_file},
    io::{self, prelude::*, BufReader, Error},
    str::from_utf8,
    time::Duration,
};

use async_std::task::sleep;
use chrono::prelude::*;
use log::debug;
use smol::{prelude::*, block_on, channel, spawn};
use smol::net::unix::UnixListener;
use smol::stream::StreamExt;
use swaybar_types::{Header, Version};
use swayipc_async::{
    Connection,
    Event,
    EventType,
    Fallible,
    Node,
    WindowChange,
    WindowEvent,
};

use allotropic::async_log;
use barnine::{Display, Update, backlight::{brightness_down, brightness_up}};

const BATTERY_UEVENT: &str = "/sys/class/power_supply/BAT0/uevent";
const LOG_FILE: &str = "log.txt";
const RPC_SOCK: &str = "/home/zach/barnine.sock";

fn main() {
    block_on(async_log(LOG_FILE, run()));
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
    smol::spawn(get_sway(tx.clone())).detach();
    smol::spawn(get_time(tx.clone())).detach();
    smol::spawn(get_rpc(tx.clone())).detach();

    // Update display
    Display {
        rx: Some(rx),
        ..Display::default()
    }.run().await;

    unreachable!();
}

async fn get_rpc(_tx: channel::Sender<Update>) -> Result<(), Error> {
    let _ = remove_file(RPC_SOCK);
    let listener = UnixListener::bind(RPC_SOCK)?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        match stream {
            Ok(mut stream) => {
                spawn(async move {
                    let mut buf = vec![0u8; 64];
                    if let Ok(len) = stream.read(&mut buf).await {
                        if let Ok(msg) = from_utf8(&buf[0..len]) {
                            match msg {
                                "brightness_up" => brightness_up().await.unwrap(),
                                "brightness_down" => brightness_down().await.unwrap(),
                                _ => debug!("bad cmd[{}]: >{:?}<", len, msg),
                            }
                        }
                    }
                }).detach();
            },
            Err(err) => {
                debug!("got err: {:?}", err);
            },
        }
    }
    Ok(())
}

async fn _get_volume(_tx: channel::Sender<Update>) {
    loop {
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
        let file = File::open(BATTERY_UEVENT)?;
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

        tx.send(Update::Redraw)
            .await
            .unwrap();

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
