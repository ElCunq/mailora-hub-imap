/// Unified Inbox - aggregates messages from all accounts
use axum::{extract::{State, Query}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::Row; // for try_get in dynamic row access

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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

#[derive(Debug, Deserialize)]
pub struct UnifiedQuery { pub limit: Option<u32>, pub offset: Option<u32>, pub unread_only: Option<bool>, pub folder: Option<String> }

/// GET /unified/inbox - Returns all INBOX messages from all accounts, sorted by date
pub async fn unified_inbox(
    State(pool): State<SqlitePool>,
    Query(q): Query<UnifiedQuery>
) -> Result<Json<UnifiedInboxResponse>, StatusCode> {
    let limit = q.limit.unwrap_or(100).min(500) as i64;
    let offset = q.offset.unwrap_or(0) as i64;
    let folder = q.folder.unwrap_or_else(|| "INBOX".to_string());
    let unread_filter = if q.unread_only.unwrap_or(false) { "AND (flags NOT LIKE '%\\Seen%')" } else { "" };
    let sql = format!(
        "SELECT account_id, folder, uid, message_id, subject, from_addr, to_addr, date, flags, size \
         FROM messages WHERE folder = ? {} ORDER BY date DESC LIMIT ? OFFSET ?",
        unread_filter
    );
    let messages = sqlx::query_as::<_, UnifiedMessage>(&sql)
        .bind(folder)
        .bind(limit)
        .bind(offset)
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

/// GET /unified/unread - Returns unread message counters
#[derive(Debug, Serialize)]
pub struct UnreadCounters { pub total_unread: i64, pub per_account: Vec<(String,i64)> }

pub async fn unified_unread(State(pool): State<SqlitePool>) -> Result<Json<UnreadCounters>, StatusCode> {
    let rows = sqlx::query("SELECT account_id, COUNT(*) as c FROM messages WHERE folder='INBOX' AND (flags NOT LIKE '%\\Seen%') GROUP BY account_id")
        .fetch_all(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut per_account = Vec::new();
    let mut total = 0i64;
    for r in rows { if let (Ok(acc), Ok(c)) = (r.try_get::<String,_>("account_id"), r.try_get::<i64,_>("c")) { per_account.push((acc.clone(), c)); total += c; } }
    Ok(Json(UnreadCounters { total_unread: total, per_account }))
}
