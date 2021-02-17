#[tokio::main]
async fn main() {
    println!("{:#?}", barnine::pulse::pulse_info().await);
}
