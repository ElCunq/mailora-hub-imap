use anyhow::Result;
use crate::models::account::Account;
use sqlx::SqlitePool;

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

/// Helper to rewrite cid: URLs in HTML mail bodies dynamically to target our attachment download route
pub fn rewrite_cids(html: &str, account_id: &str, uid: u32, folder: &str) -> String {
    let re = regex::Regex::new(r#"(?i)src=["']cid:([^"']+)["']"#).unwrap();
    let replacement = format!("src=\"/attachments/download?accountId={}&uid={}&part=$1&folder={}\"", 
                              urlencoding::encode(account_id), 
                              uid, 
                              urlencoding::encode(folder));
    re.replace_all(html, replacement.as_str()).to_string()
}

/// Fetch body with real-time IMAP retrieval. No caching of message body in SQLite.
pub async fn fetch_message_body(
    account: &Account,
    uid: u32,
    folder: Option<&str>,
    _pool: &SqlitePool,
    _force_refresh: bool,
) -> Result<MessageBody> {
    let folder = folder.unwrap_or("INBOX");

    // Fetch directly from Dovecot IMAP in real-time
    let fetched = crate::imap::sync::fetch_message_body_in(
        &account.imap_host,
        account.imap_port,
        &account.email,
        &account.password,
        uid,
        folder,
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("message not found"))?;

    let html_rewritten = fetched.html_body.map(|html| {
        rewrite_cids(&html, &account.id, uid, folder)
    });

    Ok(MessageBody {
        uid,
        folder: folder.to_string(),
        subject: fetched.subject,
        from: fetched.from,
        date: fetched.date,
        flags: fetched.flags,
        plain_text: fetched.body.clone(),
        html_text: html_rewritten,
        raw_size: fetched.body.len(),
    })
}

/// Stub for garbage collection (no longer needed in stateless mode)
pub async fn gc(_pool: &SqlitePool, _max_rows: i64) {}

/// Stub for prefetching (no longer needed in stateless mode)
pub async fn prefetch_recent_bodies(
    _account: &Account,
    _folder: &str,
    _limit: u32,
    _pool: &SqlitePool,
) -> Result<()> {
    Ok(())
}
