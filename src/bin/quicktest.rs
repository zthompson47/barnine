use allotropic::async_log;
use barnine::pulse::pulse_info;

#[tokio::main]
async fn main() {
    async_log("log.txt", run()).await;
}

async fn run() {
    let result = pulse_info().await;
    println!("{:#?}", result);
}
