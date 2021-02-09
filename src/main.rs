use std::time::Duration;

use async_std::task::sleep;
use chrono::prelude::*;
use log::debug;
use smol::{block_on, channel};
use swaybar_types::{Header, Version};

use allotropic::async_log;
use barnine::bar::{Display, Update};
use barnine::battery::get_battery;
use barnine::err::Res;
use barnine::pulse::get_pulse;
use barnine::rpc::get_rpc;
use barnine::sway::get_sway;

fn main() {
    block_on(async_log("log.txt", run()));
}

async fn run() {
    debug!("starting barnine");

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
    smol::spawn(get_pulse(tx.clone())).detach();

    // Update display
    Display {
        rx: Some(rx),
        ..Display::default()
    }
    .run()
    .await;

    unreachable!();
}

async fn get_time(tx: channel::Sender<Update>) -> Res<()> {
    let time_format = "%b %d %A %l:%M:%S %p";
    loop {
        let now: DateTime<Local> = Local::now();
        let fmt_now = now.format(time_format).to_string();
        tx.send(Update::Time(Some(fmt_now))).await?;
        tx.send(Update::Redraw).await?;
        sleep(Duration::from_secs(1)).await;
    }
}
