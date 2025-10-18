use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::services::message_sync_service::{sync_account_messages, sync_folder_messages, SyncStats};

/// POST /sync/:account_id - Sync all folders for an account
pub async fn sync_account(
    State(pool): State<sqlx::SqlitePool>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Get account from DB
    let account = sqlx::query_as::<_, crate::models::account::Account>(
        "SELECT * FROM accounts WHERE id = ?",
    )
    .bind(&account_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Account not found".to_string()))?
    .with_password()
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Start sync
    let stats = sync_account_messages(&pool, &account)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "account_id": account_id,
        "email": account.email,
        "stats": stats,
        "total_new": stats.iter().map(|s| s.new_messages).sum::<u32>(),
        "total_updated": stats.iter().map(|s| s.updated_messages).sum::<u32>(),
        "total_deleted": stats.iter().map(|s| s.deleted_messages).sum::<u32>(),
    })))
}

/// POST /sync/:account_id/:folder - Sync a specific folder
pub async fn sync_folder(
    State(pool): State<sqlx::SqlitePool>,
    Path((account_id, folder)): Path<(String, String)>,
) -> Result<Json<SyncStats>, (StatusCode, String)> {
    // Get account from DB
    let account = sqlx::query_as::<_, crate::models::account::Account>(
        "SELECT * FROM accounts WHERE id = ?",
    )
    .bind(&account_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Account not found".to_string()))?
    .with_password()
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Start sync
    let stats = sync_folder_messages(&pool, &account, &folder)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(stats))
}

/// GET /messages/:account_id - Get synced messages for an account
pub async fn get_messages(
    State(pool): State<sqlx::SqlitePool>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    #[derive(sqlx::FromRow, serde::Serialize)]
    struct MessageRow {
        id: i64,
        uid: i64,
        folder: String,
        subject: Option<String>,
        from_addr: Option<String>,
        to_addr: Option<String>,
        date: Option<String>,
        flags: Option<String>,
        size: Option<i64>,
        has_attachments: bool,
        synced_at: String,
    }

    let messages: Vec<MessageRow> = sqlx::query_as(
        r#"
        SELECT id, uid, folder, subject, from_addr, to_addr, date, flags, size, has_attachments, synced_at
        FROM messages
        WHERE account_id = ?
        ORDER BY date DESC
        LIMIT 100
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "account_id": account_id,
        "count": messages.len(),
        "messages": messages,
    })))
}

/// GET /messages/:account_id/:folder - Get messages from a specific folder
pub async fn get_folder_messages(
    State(pool): State<sqlx::SqlitePool>,
    Path((account_id, folder)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, String)> {
    #[derive(sqlx::FromRow, serde::Serialize)]
    struct MessageRow {
        id: i64,
        uid: i64,
        subject: Option<String>,
        from_addr: Option<String>,
        date: Option<String>,
        flags: Option<String>,
        has_attachments: bool,
    }

    let messages: Vec<MessageRow> = sqlx::query_as(
        r#"
        SELECT id, uid, subject, from_addr, date, flags, has_attachments
        FROM messages
        WHERE account_id = ? AND folder = ?
        ORDER BY date DESC
        LIMIT 100
        "#,
    )
    .bind(&account_id)
    .bind(&folder)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "account_id": account_id,
        "folder": folder,
        "count": messages.len(),
        "messages": messages,
    })))
}
