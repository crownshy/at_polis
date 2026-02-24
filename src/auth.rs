//! Authentication module for ATProto using bsky-sdk
//!
//! This module provides password-based authentication with the AT Protocol.
//! For production use, consider implementing OAuth 2.0 using the atrium-oauth crate.

use anyhow::{anyhow, Result};
use bsky_sdk::BskyAgent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Authenticated session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub did: String,
    pub handle: String,
}

/// Shared authenticated agent
pub type AuthAgent = Arc<RwLock<Option<BskyAgent>>>;

/// Login credentials
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}

/// Create a new BskyAgent and authenticate with the provided credentials
pub async fn login(identifier: &str, password: &str) -> Result<(BskyAgent, Session)> {
    let agent = BskyAgent::builder().build().await?;

    agent.login(identifier, password).await?;

    // Get session info
    let session_data = agent
        .get_session()
        .await
        .ok_or_else(|| anyhow!("No session after login"))?;

    let session = Session {
        did: session_data.did.to_string(),
        handle: session_data.handle.to_string(),
    };

    Ok((agent, session))
}

// Note: Session persistence functions removed due to API availability
// For future implementation, consider storing credentials securely
// and recreating agents as needed
