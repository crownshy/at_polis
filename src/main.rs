use atpolis::jetstream::run_consumer;
use std::time::Duration;
use tracing::error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    loop {
        if let Err(e) = run_consumer().await {
            error!("connection error: {:?}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
