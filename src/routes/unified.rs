use axum::{extract::{State, Query}, http::StatusCode, Json};
use crate::rbac::AuthUser;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use futures::StreamExt;
use serde_json::json;

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

#[derive(Debug, Deserialize)]
pub struct UnifiedQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub folder: Option<String>,
}

/// GET /unified/inbox - Returns aggregated messages from all assigned/viewable mailboxes statelessly
pub async fn unified_inbox(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Query(q): Query<UnifiedQuery>
) -> Result<Json<UnifiedInboxResponse>, StatusCode> {
    let folder = q.folder.unwrap_or_else(|| "INBOX".to_string());
    
    // Resolve which accounts/mailboxes are viewable by this user (RBAC)
    let accounts = if auth_user.role == "Admin" || auth_user.role == "SuperAdmin" {
        sqlx::query_as::<_, crate::models::account::Account>("SELECT * FROM accounts")
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // Members see only assigned mailboxes
        sqlx::query_as::<_, crate::models::account::Account>(
            "SELECT a.* FROM accounts a 
             JOIN user_accounts ua ON a.id = ua.account_id 
             WHERE ua.user_id = ?"
        )
        .bind(auth_user.id)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let mut tasks = Vec::new();
    for acc in accounts {
        let folder_clone = folder.clone();
        let limit_val = q.limit.unwrap_or(30).min(50) as u32;
        tasks.push(tokio::spawn(async move {
            let mut acc_dec = acc.clone();
            if acc_dec.password.is_empty() {
                if let Ok(a) = acc_dec.clone().with_password() { acc_dec = a; } else { return vec![]; }
            }
            if acc_dec.password.is_empty() { return vec![]; }

            let mut imap = match crate::imap::conn::connect(&acc_dec.imap_host, acc_dec.imap_port, &acc_dec.email, &acc_dec.password).await {
                Ok(c) => c,
                Err(_) => return vec![],
            };

            let folder_meta = match imap.session.select(&folder_clone).await {
                Ok(meta) => meta,
                Err(_) => {
                    let _ = imap.session.logout().await;
                    return vec![];
                }
            };

            if folder_meta.exists == 0 {
                let _ = imap.session.logout().await;
                return vec![];
            }

            let start = if folder_meta.exists > limit_val { folder_meta.exists - limit_val + 1 } else { 1 };
            let range = format!("{}:{}", start, folder_meta.exists);

            let mut fetches = match imap.session.fetch(&range, "UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE").await {
                Ok(f) => f,
                Err(_) => return vec![],
            };

            let mut msgs = Vec::new();
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
                    let to = env
                        .and_then(|e| e.to.as_ref())
                        .and_then(|v| v.get(0))
                        .map(|addr| crate::imap::sync::format_address(addr));
                    let date = f.internal_date().map(|d| d.to_rfc3339());
                    let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect();
                    let flags_str = serde_json::to_string(&flags).unwrap_or("[]".to_string());

                    msgs.push(UnifiedMessage {
                        account_id: acc_dec.id.clone(),
                        folder: folder_clone.clone(),
                        uid,
                        message_id: None,
                        subject,
                        from_addr: from,
                        to_addr: to,
                        date,
                        flags: Some(flags_str),
                        size: f.size.map(|s| s as i64),
                    });
                }
            }
            drop(fetches);
            let _ = imap.session.logout().await;
            msgs
        }));
    }

    let mut merged = Vec::new();
    for t in tasks {
        if let Ok(res) = t.await {
            merged.extend(res);
        }
    }

    // Sort by date descending (using simple string comparison on ISO-8601 date, or timestamp)
    merged.sort_by(|a, b| {
        let a_date = a.date.as_deref().unwrap_or("");
        let b_date = b.date.as_deref().unwrap_or("");
        b_date.cmp(a_date)
    });

    let total = merged.len();
    Ok(Json(UnifiedInboxResponse { messages: merged, total }))
}

/// GET /unified/events - Stub/Compat
#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<String>,
    pub total: usize,
}
pub async fn unified_events() -> Result<Json<EventsResponse>, StatusCode> {
    Ok(Json(EventsResponse { events: vec![], total: 0 }))
}

/// GET /unified/unread - Returns unread message counters statelessly
#[derive(Debug, Serialize)]
pub struct UnreadCounters {
    pub total_unread: i64,
    pub per_account: Vec<(String, i64)>,
}

pub async fn unified_unread(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
) -> Result<Json<UnreadCounters>, StatusCode> {
    let accounts = if auth_user.role == "Admin" || auth_user.role == "SuperAdmin" {
        sqlx::query_as::<_, crate::models::account::Account>("SELECT * FROM accounts")
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, crate::models::account::Account>(
            "SELECT a.* FROM accounts a 
             JOIN user_accounts ua ON a.id = ua.account_id 
             WHERE ua.user_id = ?"
        )
        .bind(auth_user.id)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let mut per_account = Vec::new();
    let mut total_unread = 0i64;

    for acc in accounts {
        let mut acc_dec = acc.clone();
        if acc_dec.password.is_empty() {
            if let Ok(a) = acc_dec.clone().with_password() { acc_dec = a; } else { continue; }
        }
        if acc_dec.password.is_empty() { continue; }

        if let Ok(mut imap) = crate::imap::conn::connect(&acc_dec.imap_host, acc_dec.imap_port, &acc_dec.email, &acc_dec.password).await {
            if let Ok(folder_meta) = imap.session.select("INBOX").await {
                // Fetch unread count directly via IMAP SEARCH UNSEEN
                if let Ok(unseen_uids) = imap.session.uid_search("UNSEEN").await {
                    let count = unseen_uids.len() as i64;
                    per_account.push((acc_dec.id.clone(), count));
                    total_unread += count;
                }
            }
            let _ = imap.session.logout().await;
        }
    }

    Ok(Json(UnreadCounters { total_unread, per_account }))
}
