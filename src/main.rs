use std::time::Duration;

use chrono::prelude::*;
use swaybar_types::{Header, Version};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::sleep;
use tracing::{debug, instrument};

use barnine::{
    bar::{Display, Update},
    battery::get_battery,
    err::Res,
    logging::init_logging,
    pulse::get_pulse,
    rpc::get_rpc,
    sway::get_sway,
};

#[tokio::main]
async fn main() {
    let _guard = init_logging("barnine.log");

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
    let (tx, rx) = unbounded_channel();
    spawn(get_battery(tx.clone()));
    spawn(get_sway(tx.clone()));
    spawn(get_time(tx.clone()));
    spawn(get_rpc(tx.clone()));
    spawn(get_pulse(tx.clone()));

    // Update display
    Display {
        rx: Some(rx),
        ..Display::default()
    }
    .run()
    .await;

    unreachable!();
}

#[instrument]
async fn get_time(tx: UnboundedSender<Update>) -> Res<()> {
    debug!("START get_time");
    let time_format = "%b %d %A %l:%M:%S %p";
    loop {
        debug!("LOOP get_time");
        let now: DateTime<Local> = Local::now();
        let fmt_now = now.format(time_format).to_string();
        debug!("LOOP 222222222222233");
        tx.send(Update::Time(Some(fmt_now)))?;
        debug!("LOOP 333333333333333");
        tx.send(Update::Redraw)?;
        debug!("LOOP BEFORE sleep");
        sleep(Duration::from_secs(1)).await;
        debug!("LOOP AFTER sleep");
    }
}
