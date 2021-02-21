use chrono::prelude::*;
use futures::stream::StreamExt;
use swaybar_types::{Header, Version};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::{self, Duration};

use barnine::{
    bar::{Bar, Update},
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

    let (tx_updates, mut rx_updates) = unbounded_channel();
    let futures_stream = futures::stream::iter(vec![
        spawn(get_battery(tx_updates.clone())),
        spawn(get_sway(tx_updates.clone())),
        spawn(get_time(tx_updates.clone())),
        spawn(get_rpc(tx_updates.clone())),
        spawn(get_pulse(tx_updates.clone())),
    ]);

    // Log worker failures
    let mut worker_errors = futures_stream.buffer_unordered(5);
    tokio::spawn(async move {
        while let Some(error) = worker_errors.next().await {
            tracing::error!("{:?}", error);
        }
    });

    let (tx_output, mut rx_output) = unbounded_channel();
    let mut bar = Bar::new(tx_output);

    // Process worker update messages
    tokio::spawn(async move {
        while let Some(command) = rx_updates.recv().await {
            bar.update(command);
        }
    });

    // Send redraws to output
    while let Some(json) = rx_output.recv().await {
        println!("{},", json);
    }

    unreachable!()
}

#[tracing::instrument]
async fn get_time(tx: UnboundedSender<Update>) -> Res<()> {
    tracing::trace!("START get_time");
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
