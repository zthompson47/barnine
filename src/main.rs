use chrono::prelude::*;
use futures::stream::StreamExt;
use serde_json::Value;
use swaybar_types::{Header, Version};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::{self, Duration};
use tracing::{debug, error, instrument, trace};

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
    let _guard = init_logging("barnine");

    // `man swaybar-protocol`
    let header = Header {
        version: Version::One,
        stop_signal: None,
        cont_signal: None,
        click_events: None,
    };
    println!("{}", serde_json::to_string(&header).unwrap());

    // Begin infinite json array of updates
    println!("[");

    let (tx_workers, mut rx_workers) = unbounded_channel();
    let futures_stream = futures::stream::iter(vec![
        spawn(get_battery(tx_workers.clone())),
        spawn(get_sway(tx_workers.clone())),
        spawn(get_time(tx_workers.clone())),
        spawn(get_rpc(tx_workers.clone())),
        spawn(get_pulse(tx_workers.clone())),
    ]);
    let mut worker_errors = futures_stream.buffer_unordered(5); // TODO un-hardcode?

    // Log worker failures
    tokio::spawn(async move {
        while let Some(error) = worker_errors.next().await {
            error!("{:?}", error);
        }
    });

    let (tx_display, mut rx_display) = unbounded_channel();
    let mut display = Display::new(tx_display);

    // Process worker update messages
    tokio::spawn(async move {
        while let Some(command) = rx_workers.recv().await {
            display.update(command);
        }
    });

    // Send redraws to output
    while let Some(json) = rx_display.recv().await {
        let v: Result<Value, _> = serde_json::from_str(&json);
        match v {
            Ok(_) => {
                println!("{},", json);
            },
            Err(e) => debug!("Bad json: {}", e.to_string()),
        }
    }

    unreachable!()
}

#[instrument]
async fn get_time(tx: UnboundedSender<Update>) -> Res<()> {
    trace!("START get_time");
    let time_format = "%b %d %A %l:%M:%S %p";
    let mut interval = time::interval(Duration::from_millis(1_000));

    loop {
        interval.tick().await;

        let now: DateTime<Local> = Local::now();
        let fmt_now = now.format(time_format).to_string();

        tx.send(Update::Time(Some(fmt_now)))?;
        tx.send(Update::Redraw)?;
    }
}
