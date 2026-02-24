use atpolis::{jetstream::run_consumer, start_server, AppState};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {
        tracked_tags: Arc::new(Mutex::new(vec![
            "ai".into(),
            "space".into(),
            "funny".into(),
            "EconSky".into(),
            "Ornithology".into(),
            "nsfw".into(),
        ])),
        tag_mentions: Arc::new(Mutex::new(HashMap::new())),
    });

    let state_for_stream = state.clone();
    let stream_future = async move {
        loop {
            if let Err(e) = run_consumer(state_for_stream.clone()).await {
                error!("connection error: {:?}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    };

    tokio::join!(start_server(state), stream_future);
}
