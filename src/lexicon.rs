use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum JetstreamEvent {
    #[serde(rename = "commit")]
    Commit { commit: CommitData },

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommitData {
    pub collection: String,
    pub rkey: String,
    pub repo: Option<String>,
    pub record: Value, // use your typed struct later
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    #[serde(alias = "$type")]
    pub feature_type: String,
    pub tag: Option<String>,
}
