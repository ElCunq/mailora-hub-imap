/// Unified Inbox - aggregates messages from all accounts
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub account_id: String,
    pub folder: String,
    pub uid: i64,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from_addr: Option<String>,
    pub to_addr: Option<String>,
    pub date: Option<String>,
    pub flags: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct UnifiedInboxResponse {
    pub messages: Vec<UnifiedMessage>,
    pub total: usize,
}

/// GET /unified/inbox - Returns all INBOX messages from all accounts, sorted by date
pub async fn unified_inbox(
    State(pool): State<SqlitePool>,
) -> Result<Json<UnifiedInboxResponse>, StatusCode> {
    let messages = sqlx::query_as!(
        UnifiedMessage,
        r#"
        SELECT 
            account_id, folder, uid, message_id, subject, 
            from_addr, to_addr, date, flags, size
        FROM messages
        WHERE folder = 'INBOX'
        ORDER BY date DESC
        LIMIT 100
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch unified inbox: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = messages.len();
    Ok(Json(UnifiedInboxResponse { messages, total }))
}

/// GET /unified/events - Returns recent events (IN/OUT)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventRecord {
    pub id: i64,
    pub direction: String,
    pub mailbox: String,
    pub actor: Option<String>,
    pub peer: Option<String>,
    pub subject: Option<String>,
    pub ts: i64,
}

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<EventRecord>,
    pub total: usize,
}

pub async fn unified_events(
    State(pool): State<SqlitePool>,
) -> Result<Json<EventsResponse>, StatusCode> {
    let events = sqlx::query_as!(
        EventRecord,
        r#"
        SELECT id as "id!", direction as "direction!", mailbox as "mailbox!", 
               actor, peer, subject, ts as "ts!"
        FROM events
        ORDER BY ts DESC
        LIMIT 100
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch events: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = events.len();
    Ok(Json(EventsResponse { events, total }))
}
