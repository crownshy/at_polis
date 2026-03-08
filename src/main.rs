use atpolis::{jetstream::run_consumer, start_server, PolisAppState};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize database
    let db_path = std::env::current_dir().unwrap().join("at_polis.db");
    tracing::info!("Database path: {}", db_path.display());

    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = match atpolis::db::init_db(&db_url).await {
        Ok(connection) => {
            tracing::info!("Database initialized successfully");
            connection
        }
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            panic!("Cannot start server without database: {}", e);
        }
    };

    // Initialize OAuth client
    let oauth_client = match atpolis::oauth2::create_oauth_client().await {
        Ok(client) => {
            tracing::info!("OAuth client initialized successfully");
            Arc::new(client)
        }
        Err(e) => {
            tracing::error!("Failed to initialize OAuth client: {}", e);
            panic!("Cannot start server without OAuth client: {}", e);
        }
    };

    let state = Arc::new(PolisAppState {
        tracked_tags: Arc::new(Mutex::new(vec![
            "ai".into(),
            "space".into(),
            "funny".into(),
            "EconSky".into(),
            "Ornithology".into(),
        ])),
        tag_mentions: Arc::new(Mutex::new(HashMap::new())),
        oauth_client,
        oauth_agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        db,
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
