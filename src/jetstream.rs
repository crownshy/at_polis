use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;
use url::Url;
use zstd::stream::decode_all;

use crate::lexicon::{CommitData, Feature, JetstreamEvent};

const JETSTREAM_URL: &str = "wss://jetstream1.us-east.bsky.network/subscribe";

async fn handle_message(bytes: Vec<u8>) -> anyhow::Result<()> {
    // Try decompressing (Jetstream often uses zstd)
    let data = match decode_all(&bytes[..]) {
        Ok(decompressed) => decompressed,
        Err(_) => bytes, // not compressed
    };

    handle_text(data).await
}

fn extract_hash_tags(post: CommitData) -> Vec<String> {
    let mut tags: Vec<String> = vec![];
    if let Some(facets) = post
        .record
        .get("facets")
        .and_then(serde_json::Value::as_array)
    {
        for facet in facets {
            info!("facets {facet:#?}");
            if let Some(features) = facet.get("features").and_then(serde_json::Value::as_array) {
                for feature in features.into_iter() {
                    let feature: Feature = serde_json::from_value(feature.clone()).unwrap();
                    if (feature.feature_type == "app.bsky.richtext.facet#tag") {
                        tags.push(feature.tag.unwrap());
                    }
                }
            }
        }
    };
    tags
}

async fn handle_text(data: Vec<u8>) -> anyhow::Result<()> {
    let value: Result<JetstreamEvent, _> = serde_json::from_slice(&data);

    if let Ok(message) = value {
        if let JetstreamEvent::Commit { commit } = message {
            if (commit.collection == "app.bsky.feed.post") {
                let text = commit.record.get("text");
                let tags = extract_hash_tags(commit.clone());
                info!("Post text: {text:#?} tags: {tags:#?}");
            }
        }
    }

    Ok(())
}

pub async fn run_consumer() -> anyhow::Result<()> {
    let url = Url::parse(JETSTREAM_URL)?;
    let (ws_stream, _) = connect_async(url).await?;
    info!("connected to jetstream");

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;

        match msg {
            Message::Binary(bytes) => {
                handle_message(bytes).await?;
            }
            Message::Text(text) => {
                handle_text(text.into_bytes()).await?;
            }
            Message::Close(_) => {
                info!("connection closed");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
