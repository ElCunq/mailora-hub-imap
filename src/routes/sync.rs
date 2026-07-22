use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use serde::Deserialize;
use axum::extract::Query;
use crate::rbac::AuthUser;

#[derive(Debug, serde::Serialize)]
pub struct SyncStats {
    pub folder: String,
    pub new_messages: u32,
    pub updated_messages: u32,
    pub deleted_messages: u32,
}

/// POST /sync/:account_id - Stub/No-op for stateless architecture
pub async fn sync_account(
    State(_pool): State<sqlx::SqlitePool>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(Json(json!({
        "account_id": account_id,
        "status": "noop_stateless",
        "total_new": 0,
        "total_updated": 0,
        "total_deleted": 0,
    })))
}

/// POST /sync/:account_id/:folder - Stub/No-op for stateless architecture
pub async fn sync_folder(
    State(_pool): State<sqlx::SqlitePool>,
    Path((_account_id, folder)): Path<(String, String)>,
) -> Result<Json<SyncStats>, (StatusCode, String)> {
    Ok(Json(SyncStats {
        folder,
        new_messages: 0,
        updated_messages: 0,
        deleted_messages: 0,
    }))
}

/// POST /sync/:account_id/backfill-attachments - Stub/No-op for stateless architecture
pub async fn backfill_attachments_endpoint(
    State(_pool): State<sqlx::SqlitePool>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(Json(json!({
        "account_id": account_id,
        "status": "noop_stateless"
    })))
}

/// GET /messages/:account_id/:folder - Get messages from a specific folder statelessly from Dovecot
#[derive(Debug, Deserialize)]
pub struct PageQs {
    pub limit: Option<u32>,
    pub before_uid: Option<i64>,
    pub unread: Option<bool>,
}

pub async fn get_folder_messages(
    State(pool): State<sqlx::SqlitePool>,
    Path((account_id, folder)): Path<(String, String)>,
    Query(q): Query<PageQs>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = crate::services::account_service::get_account(&pool, &account_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    let acc_with_pass = account.clone().with_password().ok();
    let password = acc_with_pass.as_ref().map(|a| a.password.clone()).unwrap_or_default();

    let host = if account.imap_host.trim().is_empty() || account.imap_host == "localhost" || account.imap_host == "127.0.0.1" {
        std::env::var("MAILCOW_IMAP_HOST").unwrap_or_else(|_| "10.0.1.1".to_string())
    } else {
        account.imap_host.clone()
    };

    if password.is_empty() {
        return Ok(Json(json!({
            "account_id": account_id,
            "folder": folder,
            "count": 0,
            "messages": [],
        })));
    }

    // Connect to local IMAP/Dovecot
    let mut imap = match crate::imap::conn::connect(&host, account.imap_port, &account.email, &password).await {
        Ok(c) => c,
        Err(_) => {
            return Ok(Json(json!({
                "account_id": account_id,
                "folder": folder,
                "count": 0,
                "messages": [],
            })));
        }
    };
    let session = &mut imap.session;

    let folder_meta = match session.select(&folder).await {
        Ok(m) => m,
        Err(_) => {
            let _ = session.logout().await;
            return Ok(Json(json!({
                "account_id": account_id,
                "folder": folder,
                "count": 0,
                "messages": [],
            })));
        }
    };

    if folder_meta.exists == 0 {
        let _ = session.logout().await;
        return Ok(Json(json!({
            "account_id": account_id,
            "folder": folder,
            "count": 0,
            "messages": [],
        })));
    }

    let limit = q.limit.unwrap_or(50).min(100) as u32;
    let exists = folder_meta.exists;
    let start = if exists > limit { exists - limit + 1 } else { 1 };
    let seq_range = format!("{}:{}", start, exists);

    use futures::StreamExt;
    let mut fetches = session.fetch(&seq_range, "UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE").await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut messages = Vec::new();
    while let Some(item) = fetches.next().await {
        if let Ok(f) = item {
            let uid = f.uid.unwrap_or(0) as i64;
            if let Some(before) = q.before_uid {
                if uid >= before {
                    continue;
                }
            }

            let env = f.envelope();
            let subject = env
                .and_then(|e| e.subject.as_ref())
                .map(|b| crate::imap::sync::decode_subject(b));
            let from = env
                .and_then(|e| e.from.as_ref())
                .and_then(|v| v.get(0))
                .map(|addr| crate::imap::sync::format_address(addr));
            let date = f.internal_date().map(|d| d.to_rfc3339());
            let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect();
            let flags_str = serde_json::to_string(&flags).unwrap_or("[]".to_string());

            if q.unread.unwrap_or(false) {
                let is_seen = flags.iter().any(|fl| fl.eq_ignore_ascii_case("\\Seen") || fl.eq_ignore_ascii_case("seen"));
                if is_seen {
                    continue;
                }
            }

            messages.push(json!({
                "id": uid,
                "uid": uid,
                "subject": subject,
                "from_addr": from,
                "date": date,
                "flags": flags_str,
                "has_attachments": false,
            }));
        }
    }

    messages.sort_by(|a, b| {
        let a_uid = a["uid"].as_i64().unwrap_or(0);
        let b_uid = b["uid"].as_i64().unwrap_or(0);
        b_uid.cmp(&a_uid)
    });

    drop(fetches);
    let _ = session.logout().await;

    Ok(Json(json!({
        "account_id": account_id,
        "folder": folder,
        "count": messages.len(),
        "messages": messages,
    })))
}

/// GET /messages/:account_id - Stub/Compat endpoint
pub async fn get_messages(
    State(pool): State<sqlx::SqlitePool>,
    Path(account_id): Path<String>,
    Query(q): Query<PageQs>,
) -> Result<Json<Value>, (StatusCode, String)> {
    get_folder_messages(State(pool), Path((account_id, "INBOX".to_string())), Query(q)).await
}

/// GET /search - Search messages statelessly using IMAP SEARCH
#[derive(Debug, Deserialize)]
pub struct SearchQs {
    pub q: Option<String>,
    pub folder: Option<String>,
    pub account_id: Option<String>,
    pub limit: Option<u32>,
}

pub async fn search_messages(
    State(pool): State<sqlx::SqlitePool>,
    _auth_user: AuthUser,
    Query(qs): Query<SearchQs>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account_id = qs.account_id.as_deref().unwrap_or("1");
    let account = crate::services::account_service::get_account(&pool, account_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Account not found".to_string()))?
        .with_password()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let query_str = qs.q.as_deref().unwrap_or("");
    if query_str.is_empty() {
        return Ok(Json(json!({
            "count": 0,
            "messages": []
        })));
    }

    let mut imap = crate::imap::conn::connect(&account.imap_host, account.imap_port, &account.email, &account.password)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let session = &mut imap.session;
    let folder = qs.folder.as_deref().unwrap_or("INBOX");
    session.select(folder).await.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // Perform raw IMAP search
    let imap_query = format!("OR SUBJECT \"{}\" FROM \"{}\"", query_str.replace("\"", "\\\""), query_str.replace("\"", "\\\""));
    let uids_set = session.uid_search(&imap_query).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if uids_set.is_empty() {
        let _ = session.logout().await;
        return Ok(Json(json!({
            "count": 0,
            "messages": []
        })));
    }

    let mut uids_vec: Vec<u32> = uids_set.into_iter().collect();
    uids_vec.sort_by(|a, b| b.cmp(a));
    let limit = qs.limit.unwrap_or(50).min(100) as usize;
    let uids_to_fetch = if uids_vec.len() > limit { &uids_vec[0..limit] } else { &uids_vec[..] };

    let uid_list_str = uids_to_fetch.iter().map(|u| u.to_string()).collect::<Vec<String>>().join(",");
    
    use futures::StreamExt;
    let mut fetches = session.uid_fetch(&uid_list_str, "UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE").await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut messages = Vec::new();
    while let Some(item) = fetches.next().await {
        if let Ok(f) = item {
            let uid = f.uid.unwrap_or(0) as i64;
            let env = f.envelope();
            let subject = env
                .and_then(|e| e.subject.as_ref())
                .map(|b| crate::imap::sync::decode_subject(b));
            let from = env
                .and_then(|e| e.from.as_ref())
                .and_then(|v| v.get(0))
                .map(|addr| crate::imap::sync::format_address(addr));
            let date = f.internal_date().map(|d| d.to_rfc3339());
            let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect();
            let flags_str = serde_json::to_string(&flags).unwrap_or("[]".to_string());

            messages.push(json!({
                "account_id": account.id,
                "folder": folder,
                "uid": uid,
                "subject": subject,
                "from_addr": from,
                "date": date,
                "flags": flags_str,
                "has_attachments": false,
            }));
        }
    }

    messages.sort_by(|a, b| {
        let a_uid = a["uid"].as_i64().unwrap_or(0);
        let b_uid = b["uid"].as_i64().unwrap_or(0);
        b_uid.cmp(&a_uid)
    });

    drop(fetches);
    let _ = session.logout().await;

    Ok(Json(json!({
        "count": messages.len(),
        "messages": messages,
    })))
}
