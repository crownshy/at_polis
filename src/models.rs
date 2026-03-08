use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A poll/topic for deliberation in the Polis-style system
/// Lexicon: com.crown-shy.testing.poll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    /// The topic or question being discussed
    pub topic: String,

    /// Optional longer description of the poll
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Timestamp when the poll was created
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    /// Optional timestamp when the poll was closed to new submissions
    #[serde(rename = "closedAt", skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,

    pub uri: String,
    pub did: String,
    pub cid: String,
}

/// A statement in the Polis-style deliberation system
/// Lexicon: com.crown-shy.testing.statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    /// The text content of the statement
    pub text: String,

    /// Reference to the poll this statement belongs to
    pub poll: PollRef,

    /// Timestamp when the statement was created
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    pub uri: String,
    pub did: String,
    pub cid: String,
}

/// A vote on a statement in the Polis-style deliberation system
/// Lexicon: com.crown-shy.testing.vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// The vote value
    pub value: VoteValue,

    /// Reference to the statement being voted on
    pub subject: StatementRef,

    /// Reference to the poll this vote belongs to
    pub poll: PollRef,

    /// Timestamp when the vote was created
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    pub uri: String,
    pub did: String,
    pub cid: String,
}

/// The possible values for a vote
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoteValue {
    Agree,
    Disagree,
    Pass,
}

/// Reference to a poll record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollRef {
    /// AT-URI of the poll record
    pub uri: String,

    /// Content identifier of the poll record
    pub cid: String,
}

/// Reference to a statement record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementRef {
    /// AT-URI of the statement record
    pub uri: String,

    /// Content identifier of the statement record
    pub cid: String,
}
