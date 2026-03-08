use chrono::Utc;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};
use url::Url;
use zstd::stream::decode_all;

use atrium_api::app::bsky::feed::post::Record as PostRecord;
use atrium_api::app::bsky::richtext::facet::MainFeaturesItem;
use atrium_api::types::Union;

use crate::db::{upsert_poll, upsert_statement, upsert_vote};
use crate::lexicon::{JetstreamEvent, COLLECTION_POLL, COLLECTION_STATEMENT, COLLECTION_VOTE};
use crate::{AppState, Mention};

// Custom lexicon types for Polis
use crate::models::{Poll, Statement, Vote, VoteValue};

const JETSTREAM_URL: &str = "wss://jetstream1.us-east.bsky.network/subscribe?\
     wantedCollections=scot.comhairle.testingPolisPollV1&\
     wantedCollections=scot.comhairle.testingPolisStatementV1&\
     wantedCollections=scot.comhairle.testingPolisVoteV1";

async fn handle_message(
    bytes: Vec<u8>,
    tracked_tags: &Vec<String>,
    state: &AppState,
) -> Option<Vec<Mention>> {
    // Try decompressing (Jetstream often uses zstd)
    let data = match decode_all(&bytes[..]) {
        Ok(decompressed) => decompressed,
        Err(_) => bytes, // not compressed
    };

    handle_text(data, tracked_tags, state).await
}

fn extract_hash_tags(post_record: &PostRecord) -> Vec<String> {
    let mut tags: Vec<String> = vec![];

    if let Some(facets) = &post_record.facets {
        for facet in facets {
            for feature_union in &facet.features {
                // Pattern match on Union::Refs to get known feature types
                if let Union::Refs(MainFeaturesItem::Tag(tag_data)) = feature_union {
                    tags.push(tag_data.tag.clone());
                }
            }
        }
    }

    tags
}

async fn handle_text(
    data: Vec<u8>,
    tracked_tags: &Vec<String>,
    state: &AppState,
) -> Option<Vec<Mention>> {
    let value: Result<JetstreamEvent, _> = serde_json::from_slice(&data);

    if let Ok(message) = value {
        if let JetstreamEvent::Commit { did, commit, .. } = message {
            // Construct AT-URI: at://did/collection/rkey
            let uri = format!("at://{}/{}/{}", did, commit.collection, commit.rkey);
            let cid = commit.cid.clone().unwrap_or_else(|| "unknown".to_string());

            // if commit.collection == "app.bsky.feed.post" {
            //     // Deserialize to strongly-typed PostRecord
            //     match serde_json::from_value::<PostRecord>(commit.record) {
            //         Ok(post_record) => {
            //             let tags = extract_hash_tags(&post_record);
            //             // if !tags.is_empty() {
            //             //     info!("Tags: {tags:#?}");
            //             // }
            //             let selected_tags: Vec<Mention> = tags
            //                 .iter()
            //                 .filter(|t| tracked_tags.contains(&t.to_lowercase()))
            //                 .map(|t| Mention {
            //                     tag: t.clone().to_lowercase(),
            //                     text: post_record.text.clone(),
            //                     by: did.clone(),
            //                     at: Utc::now(),
            //                 })
            //                 .collect();
            //
            //             if selected_tags.is_empty() {
            //                 return None;
            //             } else {
            //                 info!("Got mentions {selected_tags:#?}");
            //                 return Some(selected_tags);
            //             }
            //         }
            //         Err(e) => {
            //             info!("Failed to deserialize post record: {}", e);
            //             return None;
            //         }
            //     }
            // }
            // Handle custom Polis lexicon records
            if commit.collection == COLLECTION_POLL {
                if let Ok(poll) = serde_json::from_value::<Poll>(commit.record) {
                    info!("New poll created: {} ({})", poll.topic, uri);
                    if let Err(e) = upsert_poll(&state.db, &uri, &cid, &did, &poll).await {
                        info!("Failed to store poll in database: {}", e);
                    }
                }
            } else if commit.collection == COLLECTION_STATEMENT {
                if let Ok(statement) = serde_json::from_value::<Statement>(commit.record) {
                    info!("New statement: {} ({})", statement.text, uri);
                    if let Err(e) = upsert_statement(&state.db, &uri, &cid, &did, &statement).await
                    {
                        info!("Failed to store statement in database: {}", e);
                    }
                }
            } else if commit.collection == COLLECTION_VOTE {
                info!("processing vote {:#?}", commit.record);
                match serde_json::from_value::<Vote>(commit.record) {
                    Ok(vote) => {
                        let vote_str = match vote.value {
                            VoteValue::Agree => "agree",
                            VoteValue::Disagree => "disagree",
                            VoteValue::Pass => "pass",
                        };
                        info!("Vote: {} ({})", vote_str, uri);
                        if let Err(e) = upsert_vote(&state.db, &uri, &cid, &did, &vote).await {
                            info!("Failed to store vote in database: {}", e);
                        }
                    }
                    Err(e) => error!("Failed to deserialize vote {e:#?}"),
                }
            }
            return None;
        } else {
            return None;
        }
    } else {
        return None;
    }
}

pub async fn run_consumer(state: AppState) -> anyhow::Result<()> {
    let url = Url::parse(JETSTREAM_URL)?;
    let (ws_stream, _) = connect_async(url).await?;
    info!("connected to jetstream");

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;

        let tracked_tags = {
            let tags = state.tracked_tags.lock().await;
            tags.clone()
        };

        match msg {
            Message::Binary(bytes) => {
                if let Some(mentions) = handle_message(bytes, &tracked_tags, &state).await {
                    let mut tag_mentions = state.tag_mentions.lock().await;
                    for mention in mentions.into_iter() {
                        (*tag_mentions)
                            .entry(mention.tag.clone())
                            .or_insert_with(Vec::new)
                            .push(mention);
                    }
                }
            }
            Message::Text(text) => {
                if let Some(mentions) = handle_text(text.into_bytes(), &tracked_tags, &state).await
                {
                    let mut tag_mentions = state.tag_mentions.lock().await;
                    for mention in mentions.into_iter() {
                        (*tag_mentions)
                            .entry(mention.tag.clone())
                            .or_insert_with(Vec::new)
                            .push(mention);
                    }
                }
            }
            Message::Close(_) => {
                info!("connection closed");
                break;
            }
            Message::Ping(_) => {
                // info!("got ping");
            }
            Message::Pong(_) => {
                // info!("got pong");
            }
            Message::Frame(_) => {
                info!("got frame");
            }
        }
    }

    Ok(())
}
