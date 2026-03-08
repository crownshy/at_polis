use atrium_api::types::string::Did;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_sessions::{MemoryStore, Session as TowerSession, SessionManagerLayer};

use crate::{
    db::{get_poll, get_polls, get_statements_for_poll, next_statements_for_user_on_poll},
    lexicon::{COLLECTION_POLL, COLLECTION_STATEMENT, COLLECTION_VOTE},
    models::{Poll, Statement},
};

pub mod db;
pub mod jetstream;
pub mod lexicon;
pub mod models;
pub mod oauth2;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mention {
    tag: String,
    text: String,
    by: String,
    at: DateTime<Utc>,
}

pub struct PolisAppState {
    pub tracked_tags: Arc<Mutex<Vec<String>>>,
    pub tag_mentions: Arc<Mutex<HashMap<String, Vec<Mention>>>>,
    pub oauth_client: Arc<oauth2::ConfiguredOAuthClient>,
    // Store OAuth agents by session ID
    pub oauth_agents: Arc<
        tokio::sync::RwLock<
            HashMap<String, Arc<atrium_api::agent::Agent<oauth2::ConfiguredOAuthSession>>>,
        >,
    >,
    // Database connection
    pub db: sea_orm::DatabaseConnection,
}

pub type AppState = Arc<PolisAppState>;

#[derive(Deserialize)]
struct AddTagRequest {
    tag: String,
}

#[derive(Serialize)]
struct TagsResponse {
    tags: Vec<String>,
}

async fn get_tags(State(state): State<AppState>) -> Json<TagsResponse> {
    let tags = state.tracked_tags.lock().await;
    Json(TagsResponse { tags: tags.clone() })
}

async fn add_tag(State(state): State<AppState>, Json(payload): Json<AddTagRequest>) -> StatusCode {
    let mut tags = state.tracked_tags.lock().await;
    if !tags.contains(&payload.tag) {
        tags.push(payload.tag);
        StatusCode::CREATED
    } else {
        StatusCode::OK
    }
}

async fn remove_tag(State(state): State<AppState>, Path(tag): Path<String>) -> StatusCode {
    let mut tags = state.tracked_tags.lock().await;
    if let Some(pos) = tags.iter().position(|t| t == &tag) {
        tags.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TagReport {
    pub mentions: Vec<Mention>,
}

// Helper function to get OAuth agent from session
async fn get_oauth_agent(
    state: &AppState,
    session: &TowerSession,
) -> Option<Arc<atrium_api::agent::Agent<oauth2::ConfiguredOAuthSession>>> {
    // Get session ID
    let session_id: String = session.get("session_id").await.ok().flatten()?;

    // Get agent from storage
    let agents = state.oauth_agents.read().await;
    agents.get(&session_id).cloned()
}

// OAuth endpoints

#[derive(Deserialize)]
struct OAuthInitRequest {
    handle: String,
}

#[derive(Serialize)]
struct OAuthInitResponse {
    authorization_url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

async fn oauth_init_handler(
    State(state): State<AppState>,
    Json(payload): Json<OAuthInitRequest>,
) -> Result<Json<OAuthInitResponse>, (StatusCode, Json<ErrorResponse>)> {
    use atrium_oauth::{AuthorizeOptions, KnownScope, Scope};

    let options = AuthorizeOptions {
        scopes: vec![
            Scope::Known(KnownScope::Atproto),
            Scope::Known(KnownScope::TransitionGeneric),
            Scope::Unknown("repo:scot.comhairle.testingPolisPollV1?action=create".into()),
        ],
        ..AuthorizeOptions::default()
    };

    match state.oauth_client.authorize(&payload.handle, options).await {
        Ok(authorization_url) => {
            tracing::info!(
                "OAuth authorization initiated for handle: {}",
                payload.handle
            );
            Ok(Json(OAuthInitResponse { authorization_url }))
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            tracing::error!("Failed to initiate OAuth authorization: {}", error_msg);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "oauth_init_failed".to_string(),
                    message: error_msg,
                }),
            ))
        }
    }
}

async fn oauth_callback_handler(
    State(state): State<AppState>,
    session: TowerSession,
    axum::extract::RawQuery(query): axum::extract::RawQuery,
) -> Result<Redirect, Redirect> {
    let query_str = match query {
        Some(q) => q,
        None => {
            tracing::error!("No query parameters in OAuth callback");
            return Err(Redirect::to("http://127.0.0.1:5173/?error=missing_params"));
        }
    };

    tracing::info!("OAuth callback received with query: {}", query_str);

    // Handle the OAuth callback and get an authenticated agent
    match oauth2::handle_redirect(&state.oauth_client, &query_str).await {
        Ok(agent) => {
            let did = agent.did().await.clone();
            tracing::info!("OAuth authentication successful for DID: {did:#?}");

            // Get or create a session ID
            let session_id = match session.id() {
                Some(id) => id.to_string(),
                None => {
                    // Generate a new session if one doesn't exist
                    let new_id = uuid::Uuid::new_v4().to_string();
                    tracing::info!("Generated new session ID: {}", new_id);
                    new_id
                }
            };

            // Store the agent in our agent storage
            {
                let mut agents = state.oauth_agents.write().await;
                agents.insert(session_id.clone(), Arc::new(agent));
                tracing::info!("Stored OAuth agent for session: {}", session_id);
            }

            // Store DID in session
            session.insert("did", did.clone()).await.map_err(|e| {
                tracing::error!("Failed to store DID in session: {}", e);
                Redirect::to("http://127.0.0.1:5173/?error=session_error")
            })?;

            // Mark this session as OAuth authenticated
            session
                .insert("oauth_authenticated", true)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to store OAuth flag in session: {}", e);
                    Redirect::to("http://127.0.0.1:5173/?error=session_error")
                })?;

            session
                .insert("session_id", session_id.clone())
                .await
                .map_err(|e| {
                    tracing::error!("Failed to store session ID: {}", e);
                    Redirect::to("http://127.0.0.1:5173/?error=session_error")
                })?;

            tracing::info!("OAuth authentication complete, redirecting to app");
            Ok(Redirect::to("http://127.0.0.1:5173/?success=true"))
        }
        Err(e) => {
            tracing::error!("OAuth callback failed: {}", e);
            Err(Redirect::to("http://127.0.0.1:5173/?error=oauth_failed"))
        }
    }
}

// Session endpoints

#[derive(Serialize)]
struct MeResponse {
    authenticated: bool,
    did: Option<Did>,
}

async fn me_handler(State(state): State<AppState>, session: TowerSession) -> Json<MeResponse> {
    let oauth_agent = get_oauth_agent(&state, &session).await;
    if let Some(agent) = oauth_agent {
        Json(MeResponse {
            authenticated: true,
            did: agent.did().await,
        })
    } else {
        Json(MeResponse {
            authenticated: false,
            did: None,
        })
    }
}

async fn logout_handler(
    State(state): State<AppState>,
    session: TowerSession,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Remove agent from storage
    if let Some(session_id) = session.get::<String>("session_id").await.ok().flatten() {
        let mut agents = state.oauth_agents.write().await;
        agents.remove(&session_id);
        tracing::info!("Removed OAuth agent for session: {}", session_id);
    }

    // Clear session
    session
        .flush()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

// Poll creation endpoint

#[derive(Deserialize)]
struct CreatePollRequest {
    topic: String,
    description: Option<String>,
}

// Statement creation endpoint

#[derive(Deserialize)]
struct CreateStatementRequest {
    text: String,
    poll_uri: String,
    poll_cid: String,
}

#[derive(Serialize)]
struct CreateStatementResponse {
    success: bool,
    message: String,
    uri: Option<String>,
    cid: Option<String>,
}

// Vote creation endpoint

#[derive(Deserialize)]
struct CreateVoteRequest {
    value: String, // "agree", "disagree", or "pass"
    statement_uri: String,
    statement_cid: String,
    poll_uri: String,
    poll_cid: String,
}

#[derive(Serialize)]
struct CreateVoteResponse {
    success: bool,
    message: String,
    uri: Option<String>,
    cid: Option<String>,
}

#[derive(Serialize)]
struct CreatePollResponse {
    success: bool,
    message: String,
    uri: Option<String>,
    cid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePoll {
    topic: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
}

async fn create_poll_handler(
    State(state): State<AppState>,
    session: TowerSession,
    Json(payload): Json<CreatePollRequest>,
) -> Result<Json<CreatePollResponse>, StatusCode> {
    use atrium_api::com::atproto::repo::create_record::InputData;
    use atrium_api::types::Unknown;

    // Get OAuth agent from session
    let agent = get_oauth_agent(&state, &session)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Create the poll record
    let poll = CreatePoll {
        topic: payload.topic,
        description: payload.description,
        created_at: Utc::now(),
        closed_at: None,
    };

    // Get DID from session
    let did_string: String = session
        .get("did")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Convert poll to Unknown type for the record
    let poll_value = serde_json::to_value(&poll).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let record: Unknown =
        serde_json::from_value(poll_value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record_data = InputData {
        collection: COLLECTION_POLL.parse().unwrap(),
        repo: did_string.parse().unwrap(),
        rkey: None,
        swap_commit: None,
        validate: Some(false),
        record,
    };

    match agent
        .api
        .com
        .atproto
        .repo
        .create_record(record_data.into())
        .await
    {
        Ok(output) => {
            let cid_str = format!("{:?}", output.data.cid); // Use Debug format for Cid
            Ok(Json(CreatePollResponse {
                success: true,
                message: "Poll created successfully".to_string(),
                uri: Some(output.data.uri.to_string()),
                cid: Some(cid_str),
            }))
        }
        Err(e) => Ok(Json(CreatePollResponse {
            success: false,
            message: format!("Failed to create poll: {}", e),
            uri: None,
            cid: None,
        })),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateStatement {
    text: String,
    poll: models::PollRef,
    created_at: DateTime<Utc>,
}

async fn create_statement_handler(
    State(state): State<AppState>,
    session: TowerSession,
    Json(payload): Json<CreateStatementRequest>,
) -> Result<Json<CreateStatementResponse>, StatusCode> {
    use atrium_api::com::atproto::repo::create_record::InputData;
    use atrium_api::types::Unknown;

    // Get OAuth agent from session
    let agent = get_oauth_agent(&state, &session)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Create the statement record
    let statement = CreateStatement {
        text: payload.text,
        poll: models::PollRef {
            uri: payload.poll_uri,
            cid: payload.poll_cid,
        },
        created_at: Utc::now(),
    };

    // Get DID from session
    let did_string: String = session
        .get("did")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Convert statement to Unknown type for the record
    let statement_value =
        serde_json::to_value(&statement).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let record: Unknown =
        serde_json::from_value(statement_value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record_data = InputData {
        collection: COLLECTION_STATEMENT.parse().unwrap(),
        repo: did_string.parse().unwrap(),
        rkey: None,
        swap_commit: None,
        validate: Some(false),
        record,
    };

    match agent
        .api
        .com
        .atproto
        .repo
        .create_record(record_data.into())
        .await
    {
        Ok(output) => {
            let cid_str = format!("{:?}", output.data.cid);
            Ok(Json(CreateStatementResponse {
                success: true,
                message: "Statement created successfully".to_string(),
                uri: Some(output.data.uri.to_string()),
                cid: Some(cid_str),
            }))
        }
        Err(e) => Ok(Json(CreateStatementResponse {
            success: false,
            message: format!("Failed to create statement: {}", e),
            uri: None,
            cid: None,
        })),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateVote {
    pub value: models::VoteValue,
    pub subject: models::StatementRef,
    poll: models::PollRef,
    created_at: DateTime<Utc>,
}

async fn create_vote_handler(
    State(state): State<AppState>,
    session: TowerSession,
    Json(payload): Json<CreateVoteRequest>,
) -> Result<Json<CreateVoteResponse>, StatusCode> {
    use atrium_api::com::atproto::repo::create_record::InputData;
    use atrium_api::types::Unknown;

    // Parse vote value
    let vote_value = match payload.value.to_lowercase().as_str() {
        "agree" => models::VoteValue::Agree,
        "disagree" => models::VoteValue::Disagree,
        "pass" => models::VoteValue::Pass,
        _ => {
            return Ok(Json(CreateVoteResponse {
                success: false,
                message: "Invalid vote value. Must be 'agree', 'disagree', or 'pass'".to_string(),
                uri: None,
                cid: None,
            }))
        }
    };

    // Get OAuth agent from session
    let agent = get_oauth_agent(&state, &session)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Create the vote record
    let vote = CreateVote {
        value: vote_value,
        subject: models::StatementRef {
            uri: payload.statement_uri,
            cid: payload.statement_cid,
        },
        poll: models::PollRef {
            uri: payload.poll_uri,
            cid: payload.poll_cid,
        },
        created_at: Utc::now(),
    };

    // Get DID from session
    let did_string: String = session
        .get("did")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Convert vote to Unknown type for the record
    let vote_value = serde_json::to_value(&vote).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let record: Unknown =
        serde_json::from_value(vote_value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record_data = InputData {
        collection: COLLECTION_VOTE.parse().unwrap(),
        repo: did_string.parse().unwrap(),
        rkey: None,
        swap_commit: None,
        validate: Some(false),
        record,
    };

    match agent
        .api
        .com
        .atproto
        .repo
        .create_record(record_data.into())
        .await
    {
        Ok(output) => {
            let cid_str = format!("{:?}", output.data.cid);
            Ok(Json(CreateVoteResponse {
                success: true,
                message: "Vote created successfully".to_string(),
                uri: Some(output.data.uri.to_string()),
                cid: Some(cid_str),
            }))
        }
        Err(e) => Ok(Json(CreateVoteResponse {
            success: false,
            message: format!("Failed to create vote: {}", e),
            uri: None,
            cid: None,
        })),
    }
}

async fn list_statements(
    State(state): State<AppState>,
    Path(poll_id): Path<String>,
) -> Result<Json<Vec<Statement>>, String> {
    let statements = get_statements_for_poll(&state.db, &poll_id)
        .await
        .map_err(|e| format!("Failed to get statements {e:#?}"))?;
    Ok(Json(statements))
}

async fn get_poll_handler(
    State(state): State<AppState>,
    Path(poll_uri): Path<String>,
) -> Result<Json<Option<Poll>>, String> {
    let poll = get_poll(&state.db, &poll_uri)
        .await
        .map_err(|e| format!("Failed to get poll {e:#?}"))?;
    Ok(Json(poll))
}

async fn list_polls(State(state): State<AppState>) -> Result<Json<Vec<Poll>>, String> {
    let polls = get_polls(&state.db)
        .await
        .map_err(|e| format!("Failed to get polls {e:#?}"))?;
    Ok(Json(polls))
}

/// Return the next satement for the current user
async fn next_statement(
    State(state): State<AppState>,
    Path(poll_id): Path<String>,
    session: TowerSession,
) -> Result<Json<Option<Statement>>, String> {
    let oauth_agent = get_oauth_agent(&state, &session).await;
    if let Some(agent) = oauth_agent {
        if let Some(did) = agent.did().await {
            let statement = next_statements_for_user_on_poll(&state.db, &poll_id, &did.to_string())
                .await
                .map_err(|e| format!("Failed to get next statement for user {e:#?}"))?;
            Ok(Json(statement))
        } else {
            Err("User not found".into())
        }
    } else {
        Err("User not found".into())
    }
}

pub async fn start_server(state: AppState) -> Result<(), anyhow::Error> {
    // Set up session store
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false); // Set to true in production with HTTPS

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        // Tag tracking endpoints
        // OAuth endpoints
        .route("/oauth/authorize", post(oauth_init_handler))
        .route("/oauth/callback", get(oauth_callback_handler))
        .route("/logout", post(logout_handler))
        .route("/me", get(me_handler))
        // Polis endpoints
        .route("/polls", post(create_poll_handler))
        .route("/polls", get(list_polls))
        .route("/polls/{poll_uri}", get(get_poll_handler))
        .route("/statements", post(create_statement_handler))
        .route("/polls/{poll_id}/statements", get(list_statements))
        .route("/polis/{poll_id}/next_statement", get(next_statement))
        .route("/votes", post(create_vote_handler))
        .layer(cors)
        .layer(session_layer)
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
