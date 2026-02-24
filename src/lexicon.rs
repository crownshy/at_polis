//! Lexicon definitions and Jetstream event types for ATProto records.
//!
//! This module contains Jetstream event handling types for processing
//! ATProto records from the firehose.
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use crate::lexicon::JetstreamEvent;
//! use crate::models::{Poll, Statement, Vote, VoteValue};
//!
//! // Deserialize from a Jetstream commit record
//! let poll: Poll = serde_json::from_value(commit.record)?;
//! println!("Poll topic: {}", poll.topic);
//! ```

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum JetstreamEvent {
    #[serde(rename = "commit")]
    Commit {
        did: String,
        time_us: u64,
        commit: CommitData,
    },

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommitData {
    pub rev: String,
    pub operation: String,
    pub collection: String,
    pub rkey: String,
    pub record: Value, // Keep as Value, deserialize based on collection type
}

// Collection identifiers for our custom lexicons
pub const COLLECTION_POLL: &str = "com.crown-shy.testing.poll";
pub const COLLECTION_STATEMENT: &str = "com.crown-shy.testing.statement";
pub const COLLECTION_VOTE: &str = "com.crown-shy.testing.vote";
