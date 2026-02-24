use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_sessions::{Session as TowerSession, MemoryStore, SessionManagerLayer};
use tower_http::cors::{CorsLayer, Any};

pub mod auth;
pub mod jetstream;
pub mod lexicon;
pub mod models;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mention {
    tag: String,
    text: String,
    by: String,
    at: DateTime<Utc>,
}

pub struct AppState {
    pub tracked_tags: Arc<Mutex<Vec<String>>>,
    pub tag_mentions: Arc<Mutex<HashMap<String, Vec<Mention>>>>,
}

pub type State = Arc<AppState>;

#[derive(Deserialize)]
struct AddTagRequest {
    tag: String,
}

#[derive(Serialize)]
struct TagsResponse {
    tags: Vec<String>,
}

async fn get_tags(AxumState(state): AxumState<State>) -> Json<TagsResponse> {
    let tags = state.tracked_tags.lock().await;
    Json(TagsResponse { tags: tags.clone() })
}

async fn add_tag(
    AxumState(state): AxumState<State>,
    Json(payload): Json<AddTagRequest>,
) -> StatusCode {
    let mut tags = state.tracked_tags.lock().await;
    if !tags.contains(&payload.tag) {
        tags.push(payload.tag);
        StatusCode::CREATED
    } else {
        StatusCode::OK
    }
}

async fn remove_tag(AxumState(state): AxumState<State>, Path(tag): Path<String>) -> StatusCode {
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

async fn get_tag_report(
    AxumState(state): AxumState<State>,
    Path(tag): Path<String>,
) -> Result<Json<TagReport>, StatusCode> {
    let tags = state.tag_mentions.lock().await;
    if let Some(mentions) = tags.get(&tag) {
        Ok(Json(TagReport {
            mentions: (*mentions).clone(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Authentication endpoints

#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    message: String,
    session: Option<auth::Session>,
}

async fn login_handler(
    session: TowerSession,
    Json(payload): Json<auth::LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    match auth::login(&payload.identifier, &payload.password).await {
        Ok((_agent, session_data)) => {
            // Store session data
            session
                .insert("did", session_data.did.clone())
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            session
                .insert("handle", session_data.handle.clone())
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Note: In production, you'd want to store the agent in a session store
            // For now, we'll recreate it on each request
            Ok(Json(LoginResponse {
                success: true,
                message: "Login successful".to_string(),
                session: Some(session_data),
            }))
        }
        Err(e) => Ok(Json(LoginResponse {
            success: false,
            message: format!("Login failed: {}", e),
            session: None,
        })),
    }
}

async fn logout_handler(session: TowerSession) -> Result<Json<serde_json::Value>, StatusCode> {
    session
        .flush()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

#[derive(Serialize)]
struct MeResponse {
    authenticated: bool,
    session: Option<auth::Session>,
}

async fn me_handler(session: TowerSession) -> Json<MeResponse> {
    let did: Option<String> = session.get("did").await.ok().flatten();
    let handle: Option<String> = session.get("handle").await.ok().flatten();

    match (did, handle) {
        (Some(did), Some(handle)) => Json(MeResponse {
            authenticated: true,
            session: Some(auth::Session { did, handle }),
        }),
        _ => Json(MeResponse {
            authenticated: false,
            session: None,
        }),
    }
}

// Poll creation endpoint

#[derive(Deserialize)]
struct CreatePollRequest {
    topic: String,
    description: Option<String>,
    #[serde(flatten)]
    credentials: auth::LoginRequest,
}

// Statement creation endpoint

#[derive(Deserialize)]
struct CreateStatementRequest {
    text: String,
    poll_uri: String,
    poll_cid: String,
    #[serde(flatten)]
    credentials: auth::LoginRequest,
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
    #[serde(flatten)]
    credentials: auth::LoginRequest,
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

async fn create_poll_handler(
    Json(payload): Json<CreatePollRequest>,
) -> Result<Json<CreatePollResponse>, StatusCode> {
    use atrium_api::com::atproto::repo::create_record::InputData;
    use atrium_api::types::Unknown;

    // Login to get authenticated agent
    let (agent, _session) = auth::login(&payload.credentials.identifier, &payload.credentials.password)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Create the poll record
    let poll = models::Poll {
        topic: payload.topic,
        description: payload.description,
        created_at: Utc::now(),
        closed_at: None,
    };

    // Get the authenticated session
    let session_data = agent
        .get_session()
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Convert poll to Unknown type for the record
    let poll_value = serde_json::to_value(&poll).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let record: Unknown = serde_json::from_value(poll_value)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record_data = InputData {
        collection: "com.crown-shy.testing.poll".parse().unwrap(),
        repo: session_data.did.clone().into(),
        rkey: None,
        swap_commit: None,
        validate: Some(true),
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

async fn create_statement_handler(
    Json(payload): Json<CreateStatementRequest>,
) -> Result<Json<CreateStatementResponse>, StatusCode> {
    use atrium_api::com::atproto::repo::create_record::InputData;
    use atrium_api::types::Unknown;

    // Login to get authenticated agent
    let (agent, _session) = auth::login(&payload.credentials.identifier, &payload.credentials.password)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Create the statement record
    let statement = models::Statement {
        text: payload.text,
        poll: models::PollRef {
            uri: payload.poll_uri,
            cid: payload.poll_cid,
        },
        created_at: Utc::now(),
    };

    // Get the authenticated session
    let session_data = agent
        .get_session()
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Convert statement to Unknown type for the record
    let statement_value = serde_json::to_value(&statement)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let record: Unknown = serde_json::from_value(statement_value)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record_data = InputData {
        collection: "com.crown-shy.testing.statement".parse().unwrap(),
        repo: session_data.did.clone().into(),
        rkey: None,
        swap_commit: None,
        validate: Some(true),
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

async fn create_vote_handler(
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

    // Login to get authenticated agent
    let (agent, _session) = auth::login(&payload.credentials.identifier, &payload.credentials.password)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Create the vote record
    let vote = models::Vote {
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

    // Get the authenticated session
    let session_data = agent
        .get_session()
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Convert vote to Unknown type for the record
    let vote_value = serde_json::to_value(&vote)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let record: Unknown = serde_json::from_value(vote_value)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record_data = InputData {
        collection: "com.crown-shy.testing.vote".parse().unwrap(),
        repo: session_data.did.clone().into(),
        rkey: None,
        swap_commit: None,
        validate: Some(true),
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

pub async fn start_server(state: State) -> Result<(), anyhow::Error> {
    // Set up session store
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false); // Set to true in production with HTTPS

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        // Tag tracking endpoints
        .route("/tags", get(get_tags))
        .route("/tags", post(add_tag))
        .route("/tags/{tag}", get(get_tag_report))
        .route("/tags/{tag}", delete(remove_tag))
        // Auth endpoints
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/me", get(me_handler))
        // Polis endpoints
        .route("/polls", post(create_poll_handler))
        .route("/statements", post(create_statement_handler))
        .route("/votes", post(create_vote_handler))
        .layer(cors)
        .layer(session_layer)
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
