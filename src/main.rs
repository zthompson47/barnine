use chrono::prelude::*;
use futures::stream::StreamExt;
use swaybar_types::{Header, Version};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::{self, Duration};

use barnine::{
    bar::{Bar, Update},
    battery::watch_battery,
    config::watch_config,
    err::Res,
    logging::init_logging,
    pulse::watch_pulse,
    rpc::watch_rpc,
    sway::watch_sway,
};

#[tokio::main(flavor = "current_thread")]
//#[tokio::main]
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

    // Begin infinite json-array of updates
    println!("[");

    // Spawn stats collecting workers
    let (tx_updates, rx_updates) = unbounded_channel();
    let futures_stream = futures::stream::iter(vec![
        spawn(watch_rpc(tx_updates.clone())),
        spawn(watch_sway(tx_updates.clone())),
        spawn(watch_time(tx_updates.clone())),
        spawn(watch_pulse(tx_updates.clone())),
        spawn(watch_config(tx_updates.clone())),
        spawn(watch_battery(tx_updates.clone())),
    ]);

    // Log worker failures
    let mut worker_errors = futures_stream.buffer_unordered(5);
    tokio::spawn(async move {
        while let Some(error) = worker_errors.next().await {
            tracing::error!("{:?}", error);
        }
    });

    // Write the bar
    let mut bar = Bar::new();
    bar.write_json(&mut std::io::stdout(), rx_updates).await;

    unreachable!()
}

async fn watch_time(tx: UnboundedSender<Update>) -> Res<()> {
    tracing::trace!("Start watch_time");
    let time_format = "%b %d %A %l:%M:%S %p";
    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let now: DateTime<Local> = Local::now();
        let fmt_time = Some(now.format(time_format).to_string());

        tx.send(Update::Time(Some(fmt_time.as_ref().unwrap().to_string())))?;
        tx.send(Update::Redraw)?;
    }
}
