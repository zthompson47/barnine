use smol::block_on;

use allotropic::async_log;
use barnine::pulse::pulse_info;

fn main() {
    block_on(async_log("log.txt", run()));
}

async fn run() {
    let result = pulse_info().await;
    println!("{:#?}", result);
}
