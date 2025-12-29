use anyhow::Result;
use crate::models::account::Account;
use sqlx::{Row, SqlitePool};
use sqlx::Sqlite; // for query_scalar generic DB type

#[derive(Debug, serde::Serialize)]
pub struct MessageBody {
    pub uid: u32,
    pub folder: String,
    pub subject: String,
    pub from: String,
    pub date: Option<String>,
    pub flags: Vec<String>,
    pub plain_text: String,
    pub html_text: Option<String>,
    pub raw_size: usize,
}

/// Fetch body with simple cache layer in `message_bodies` table.
pub async fn fetch_message_body(account: &Account, uid: u32, folder: Option<&str>, pool: &SqlitePool, force_refresh: bool) -> Result<MessageBody> {
    let folder = folder.unwrap_or("INBOX");

    // Cache lookup (skip if force_refresh)
    if !force_refresh {
        if let Ok(row_opt) = sqlx::query("SELECT body, html_body, subject, from_addr, date, flags FROM message_bodies WHERE account_id=? AND folder=? AND uid=?")
            .bind(&account.id)
            .bind(folder)
            .bind(uid as i64)
            .fetch_optional(pool)
            .await {
            if let Some(row) = row_opt {
                if let (Ok(body), Ok(html_body_opt)) = (row.try_get::<String,_>("body"), row.try_get::<Option<String>,_>("html_body")) {
                    let subject: String = row.try_get::<Option<String>,_>("subject").ok().flatten().unwrap_or_default();
                    let from: String = row.try_get::<Option<String>,_>("from_addr").ok().flatten().unwrap_or_default();
                    let date: Option<String> = row.try_get::<Option<String>,_>("date").ok().flatten();
                    let flags_json: String = row.try_get::<Option<String>,_>("flags").ok().flatten().unwrap_or_default();
                    let flags: Vec<String> = serde_json::from_str(&flags_json).unwrap_or_default();
                    return Ok(MessageBody { uid, folder: folder.to_string(), subject, from, date, flags, plain_text: body.clone(), html_text: html_body_opt, raw_size: body.len() });
                }
            }
        }
    }

    // IMAP fetch
    let fetched = crate::imap::sync::fetch_message_body_in(&account.imap_host, account.imap_port, &account.email, &account.password, uid, folder)
        .await?
        .ok_or_else(|| anyhow::anyhow!("message not found"))?;
    let body_text = fetched.body.clone();
    let mut html_opt = fetched.html_body.clone();
    let mut html_opt = fetched.html_body.clone();
    // Optimization: Do NOT sanitize here. The frontend uses a sandboxed iframe.
    // Backend sanitization was stripping essential email struct/styles causing blank screens.
    // if let Some(html) = html_opt.as_ref() { ... }
    // Best-effort cache write (ignore errors e.g., when table missing)
    let _ = sqlx::query("INSERT OR REPLACE INTO message_bodies (account_id, folder, uid, body, html_body, subject, from_addr, date, flags) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&account.id)
        .bind(folder)
        .bind(uid as i64)
        .bind(&body_text)
        .bind(&html_opt)
        .bind(&fetched.subject)
        .bind(&fetched.from)
        .bind(&fetched.date)
        .bind(serde_json::to_string(&fetched.flags).unwrap_or_else(|_| "[]".into()))
        .execute(pool)
        .await;

    Ok(MessageBody { uid, folder: folder.to_string(), subject: fetched.subject, from: fetched.from, date: fetched.date, flags: fetched.flags, plain_text: body_text.clone(), html_text: html_opt, raw_size: body_text.len() })
}

/// Garbage collect old cache entries (TTL 48h) and cap total entries to max_rows.
pub async fn gc(pool: &SqlitePool, max_rows: i64) {
    // Delete older than 48h
    let _ = sqlx::query("DELETE FROM message_bodies WHERE created_at < strftime('%s','now') - 172800").execute(pool).await;
    // Cap size
    if let Ok(Some(cnt)) = sqlx::query_scalar::<Sqlite, i64>("SELECT COUNT(*) FROM message_bodies").fetch_optional(pool).await {
        if cnt > max_rows {
            let overflow = cnt - max_rows;
            let _ = sqlx::query("DELETE FROM message_bodies WHERE rowid IN (SELECT rowid FROM message_bodies ORDER BY created_at ASC LIMIT ?)")
                .bind(overflow)
                .execute(pool)
                .await;
        }
    }
}
