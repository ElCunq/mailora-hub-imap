use anyhow::{Context, Result};
use async_imap::types::{Fetch, Flag};
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;
use tracing::{info, warn};
use tokio::time::{timeout, Duration};

use crate::imap::conn;
use crate::models::account::Account;

fn imap_timeout() -> Duration {
    let secs = std::env::var("MAILORA_IMAP_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60);
    Duration::from_secs(secs)
}

async fn with_timeout<F, T, E>(fut: F, ctx: &str) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display + Send + Sync + 'static,
{
    match timeout(imap_timeout(), fut).await {
        Ok(Ok(val)) => Ok(val),
        Ok(Err(e)) => Err(anyhow::anyhow!("{}: {}", ctx, e)),
        Err(_) => Err(anyhow::anyhow!("{}: timeout after {:?}", ctx, imap_timeout())),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncStats {
    pub account_id: String,
    pub folder: String,
    pub total_messages: u32,
    pub new_messages: u32,
    pub updated_messages: u32,
    pub deleted_messages: u32,
    pub duration_ms: u64,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackfillFolderStats {
    pub folder: String,
    pub scanned: usize,
    pub updated: usize,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackfillStats {
    pub account_id: String,
    pub total_scanned: usize,
    pub total_updated: usize,
    pub folders: Vec<BackfillFolderStats>,
}

/// Sync all messages from an account's folder to SQLite
pub async fn sync_folder_messages(
    pool: &SqlitePool,
    account: &Account,
    folder: &str,
) -> Result<SyncStats> {
    let start = std::time::Instant::now();

    info!(
        "Starting sync for account {} folder {}",
        account.email, folder
    );

    // Connect to IMAP
    let mut imap_session = with_timeout(
        conn::connect(
            &account.imap_host,
            account.imap_port,
            &account.email,
            &account.password,
        ),
        "IMAP connect",
    )
    .await?;

    let session = &mut imap_session.session;

    // Select folder
    let mailbox = with_timeout(session.select(folder), "IMAP SELECT").await?;

    let total_messages = mailbox.exists;
    info!("Folder {} has {} messages", folder, total_messages);

    if total_messages == 0 {
        return Ok(SyncStats {
            account_id: account.id.clone(),
            folder: folder.to_string(),
            total_messages: 0,
            new_messages: 0,
            updated_messages: 0,
            deleted_messages: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Get existing UIDs from database
    let existing_uids: HashSet<u32> =
        sqlx::query_scalar("SELECT uid FROM messages WHERE account_id = ? AND folder = ?")
            .bind(&account.id)
            .bind(folder)
            .fetch_all(pool)
            .await?
            .into_iter()
            .collect();

    info!("Found {} existing messages in DB", existing_uids.len());

    // Fetch all UIDs from server
    let sequence = format!("1:{}", total_messages);
    let messages = with_timeout(session.fetch(&sequence, "UID"), "IMAP FETCH UIDs").await?;

    let mut messages_vec: Vec<_> = vec![];
    {
        use futures::StreamExt;
        let mut stream = messages;
        while let Some(fetch) = stream.next().await {
            if let Ok(f) = fetch {
                messages_vec.push(f);
            }
        }
    }

    let server_uids: HashSet<u32> = messages_vec.iter().filter_map(|m| m.uid).collect();

    // Calculate what to sync
    let new_uids: Vec<u32> = server_uids.difference(&existing_uids).copied().collect();

    let deleted_uids: Vec<u32> = existing_uids.difference(&server_uids).copied().collect();

    info!(
        "Sync plan: {} new, {} to delete",
        new_uids.len(),
        deleted_uids.len()
    );

    // Delete messages that no longer exist on server
    let deleted_count = if !deleted_uids.is_empty() {
        let placeholders = deleted_uids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "DELETE FROM messages WHERE account_id = ? AND folder = ? AND uid IN ({})",
            placeholders
        );

        let mut q = sqlx::query(&query).bind(&account.id).bind(folder);

        for uid in &deleted_uids {
            q = q.bind(uid);
        }

        q.execute(pool).await?.rows_affected() as u32
    } else {
        0
    };

    // Fetch and store new messages
    let mut new_count = 0;
    let mut updated_count = 0;

    if !new_uids.is_empty() {
        // Fetch in batches of 50
        for chunk in new_uids.chunks(50) {
            let uid_set = chunk
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(",");

            // Include BODY.PEEK[] so we can parse attachments for metadata persistence
            let messages = with_timeout(
                session.uid_fetch(
                    &uid_set,
                    "(UID FLAGS ENVELOPE INTERNALDATE RFC822.SIZE BODY.PEEK[])",
                ),
                "IMAP UID FETCH",
            )
            .await
            .context("Failed to fetch message details")?;

            use futures::StreamExt;
            let mut stream = messages;
            while let Some(fetch_result) = stream.next().await {
                match fetch_result {
                    Ok(fetch) => match save_message_to_db(pool, account, folder, &fetch).await {
                        Ok(true) => new_count += 1,
                        Ok(false) => updated_count += 1,
                        Err(e) => warn!(
                            "Failed to save message UID {}: {}",
                            fetch.uid.unwrap_or(0),
                            e
                        ),
                    },
                    Err(e) => warn!("Failed to fetch message: {}", e),
                }
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    info!(
        "Sync completed in {}ms: {} new, {} updated, {} deleted",
        duration_ms, new_count, updated_count, deleted_count
    );

    Ok(SyncStats {
        account_id: account.id.clone(),
        folder: folder.to_string(),
        total_messages,
        new_messages: new_count,
        updated_messages: updated_count,
        deleted_messages: deleted_count,
        duration_ms,
    })
}

/// Extract attachments metadata from a raw email
fn extract_attachments_from_raw(raw: &[u8]) -> Vec<(Option<String>, Option<String>, i64, Option<String>, bool)> {
    let mut atts = Vec::new();
    if raw.len() > 5_000_000 { // safety cap
        return atts;
    }
    if let Ok(parsed) = mailparse::parse_mail(raw) {
        fn walk(pm: &mailparse::ParsedMail, out: &mut Vec<(Option<String>, Option<String>, i64, Option<String>, bool)>) {
            if pm.subparts.is_empty() {
                let ctype = pm.ctype.mimetype.to_lowercase();
                let mut filename: Option<String> = None;
                let mut content_id: Option<String> = None;
                let mut is_inline = false;
                for h in &pm.headers {
                    let key = h.get_key();
                    if key.eq_ignore_ascii_case("Content-ID") {
                        let v = h.get_value();
                        let norm = v.trim().trim_matches(['<','>']).to_string();
                        if !norm.is_empty() { content_id = Some(norm); }
                    } else if key.eq_ignore_ascii_case("Content-Disposition") {
                        let v = h.get_value();
                        let lower = v.to_lowercase();
                        if lower.contains("inline") { is_inline = true; }
                        // filename param
                        for tok in v.split(';') {
                            let t = tok.trim();
                            let tl = t.to_lowercase();
                            if tl.starts_with("filename=") { let val = t.splitn(2,'=').nth(1).unwrap_or("").trim().trim_matches('"'); if !val.is_empty(){ filename = Some(val.to_string()); break; } }
                        }
                    } else if key.eq_ignore_ascii_case("Content-Type") {
                        // try name=
                        let v = h.get_value();
                        for tok in v.split(';') {
                            let t = tok.trim();
                            let tl = t.to_lowercase();
                            if tl.starts_with("name=") { let val = t.splitn(2,'=').nth(1).unwrap_or("").trim().trim_matches('"'); if !val.is_empty(){ filename = Some(val.to_string()); break; } }
                        }
                    }
                }
                let is_text = ctype == "text/plain" || ctype == "text/html";
                if filename.is_some() || !is_text {
                    let sz = pm.get_body_raw().map(|b| b.len() as i64).unwrap_or(0);
                    out.push((filename, Some(ctype), sz, content_id, is_inline));
                }
                return;
            }
            for sp in &pm.subparts { walk(sp, out); }
        }
        walk(&parsed, &mut atts);
    }
    atts
}

/// Save a single message to database
async fn save_message_to_db(
    pool: &SqlitePool,
    account: &Account,
    folder: &str,
    fetch: &Fetch,
) -> Result<bool> {
    let uid = fetch.uid.context("Message has no UID")?;

    // Extract envelope data
    let envelope = fetch.envelope();
    let subject_raw = envelope
        .and_then(|e| e.subject.as_ref())
        .map_or(&[][..], |v| v);
    let subject = {
        let mut composed = b"Subject: ".to_vec();
        composed.extend_from_slice(subject_raw);
        match mailparse::parse_header(&composed) {
            Ok((h, _)) => h.get_value().trim().to_string(),
            Err(_) => String::from_utf8_lossy(subject_raw).trim().to_string(),
        }
    };

    let from = envelope
        .and_then(|e| e.from.as_ref())
        .and_then(|addrs| addrs.first())
        .and_then(|addr| {
            let mailbox_bytes = addr.mailbox.as_ref()?;
            let host_bytes = addr.host.as_ref()?;
            let mailbox = std::str::from_utf8(mailbox_bytes).ok()?;
            let host = std::str::from_utf8(host_bytes).ok()?;
            Some(format!("{}@{}", mailbox, host))
        })
        .unwrap_or_default();

    let to = envelope
        .and_then(|e| e.to.as_ref())
        .and_then(|addrs| addrs.first())
        .and_then(|addr| {
            let mailbox_bytes = addr.mailbox.as_ref()?;
            let host_bytes = addr.host.as_ref()?;
            let mailbox = std::str::from_utf8(mailbox_bytes).ok()?;
            let host = std::str::from_utf8(host_bytes).ok()?;
            Some(format!("{}@{}", mailbox, host))
        })
        .unwrap_or_default();

    let date = envelope
        .and_then(|e| e.date.as_ref())
        .and_then(|d| std::str::from_utf8(d).ok())
        .unwrap_or("");

    let message_id = envelope
        .and_then(|e| e.message_id.as_ref())
        .and_then(|id| std::str::from_utf8(id).ok())
        .unwrap_or("");

    // Extract flags
    let flags: Vec<String> = fetch
        .flags()
        .filter_map(|f| match f {
            Flag::Seen => Some("\\Seen".to_string()),
            Flag::Answered => Some("\\Answered".to_string()),
            Flag::Flagged => Some("\\Flagged".to_string()),
            Flag::Deleted => Some("\\Deleted".to_string()),
            Flag::Draft => Some("\\Draft".to_string()),
            Flag::Recent => Some("\\Recent".to_string()),
            _ => None,
        })
        .collect();

    let flags_json = serde_json::to_string(&flags)?;

    // Message size
    let size = fetch.size.unwrap_or(0) as i64;

    // Parse raw for attachments metadata (if available)
    let mut attachments: Vec<(Option<String>, Option<String>, i64, Option<String>, bool)> = Vec::new();
    if let Some(body_bytes) = fetch.body() {
        attachments = extract_attachments_from_raw(body_bytes);
    }
    let has_atts = !attachments.is_empty();

    // Check if message already exists
    let exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM messages WHERE account_id = ? AND folder = ? AND uid = ?",
    )
    .bind(&account.id)
    .bind(folder)
    .bind(uid)
    .fetch_one(pool)
    .await?;

    if exists {
        // Update flags only
        sqlx::query(
            "UPDATE messages SET flags = ?, synced_at = datetime('now') WHERE account_id = ? AND folder = ? AND uid = ?",
        )
        .bind(&flags_json)
        .bind(&account.id)
        .bind(folder)
        .bind(uid)
        .execute(pool)
        .await?;

        Ok(false) // Updated
    } else {
        // Insert new message
        sqlx::query(
            r#"
            INSERT INTO messages (
                account_id, folder, uid, message_id,
                subject, from_addr, to_addr, date,
                flags, size, has_attachments,
                synced_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
            "#,
        )
        .bind(&account.id)
        .bind(folder)
        .bind(uid)
        .bind(message_id)
        .bind(&subject)
        .bind(&from)
        .bind(&to)
        .bind(date)
        .bind(&flags_json)
        .bind(size as i64)
        .bind(has_atts)
        .execute(pool)
        .await?;

        // If we have attachments, persist them
        if has_atts {
            let msg_id: i64 = sqlx::query_scalar(
                "SELECT id FROM messages WHERE account_id = ? AND folder = ? AND uid = ?",
            )
            .bind(&account.id)
            .bind(folder)
            .bind(uid)
            .fetch_one(pool)
            .await?;

            // Clean any existing (shouldn't be any for new row)
            sqlx::query("DELETE FROM attachments WHERE message_id = ?")
                .bind(msg_id)
                .execute(pool)
                .await?;

            for (filename, content_type, sz, content_id, is_inline) in attachments {
                sqlx::query(
                    "INSERT INTO attachments (message_id, filename, content_type, size, content_id, is_inline) VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(msg_id)
                .bind(filename)
                .bind(content_type)
                .bind(sz)
                .bind(content_id)
                .bind(is_inline)
                .execute(pool)
                .await?;
            }
        }

        Ok(true) // New
    }
}

/// Sync all folders for an account
pub async fn sync_account_messages(pool: &SqlitePool, account: &Account) -> Result<Vec<SyncStats>> {
    let mut imap_session = with_timeout(
        conn::connect(
            &account.imap_host,
            account.imap_port,
            &account.email,
            &account.password,
        ),
        "IMAP connect",
    )
    .await?;

    let session = &mut imap_session.session;

    // List all folders
    let folders = with_timeout(session.list(None, Some("*")), "IMAP LIST").await?;

    let mut folder_names: Vec<String> = Vec::new();
    {
        use futures::StreamExt;
        let mut stream = folders;
        while let Some(name_result) = stream.next().await {
            if let Ok(name) = name_result {
                // name.name() returns &str directly, no need for from_utf8
                folder_names.push(name.name().to_string());
            }
        }
    }

    info!(
        "Syncing {} folders for {}",
        folder_names.len(),
        account.email
    );

    let mut stats = Vec::new();

    for folder in &folder_names {
        match sync_folder_messages(pool, account, folder).await {
            Ok(s) => stats.push(s),
            Err(e) => warn!("Failed to sync folder {}: {}", folder, e),
        }
    }

    Ok(stats)
}

/// Upsert a minimal message row for Sent APPEND results
pub async fn upsert_sent_message(
    pool: &SqlitePool,
    account: &Account,
    folder: &str,
    uid: u32,
    subject: Option<&str>,
    to: Option<&str>,
) -> Result<()> {
    let flags_json = serde_json::to_string(&vec!["\\Seen".to_string()])?;

    // Try update, else insert
    let updated = sqlx::query(
        "UPDATE messages SET subject = COALESCE(?, subject), to_addr = COALESCE(?, to_addr), flags = ?, synced_at = datetime('now') WHERE account_id = ? AND folder = ? AND uid = ?",
    )
    .bind(subject)
    .bind(to)
    .bind(&flags_json)
    .bind(&account.id)
    .bind(folder)
    .bind(uid as i64)
    .execute(pool)
    .await?
    .rows_affected();

    if updated == 0 {
        sqlx::query(
            r#"INSERT OR IGNORE INTO messages (account_id, folder, uid, subject, to_addr, flags, size, synced_at, internal_date)
               VALUES (?, ?, ?, ?, ?, ?, 0, datetime('now'), datetime('now'))"#,
        )
        .bind(&account.id)
        .bind(folder)
        .bind(uid as i64)
        .bind(subject)
        .bind(to)
        .bind(&flags_json)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Quick-scan Sent folder(s) to resolve UID of a just-sent message by Message-Id.
/// Returns Some((folder, uid)) when found.
pub async fn quick_sync_sent_and_upsert(
    pool: &SqlitePool,
    account: &Account,
    message_id: &str,
    subject: Option<&str>,
    to: Option<&str>,
    max_scan: usize,
) -> Result<Option<(String, u32)>> {
    use futures::StreamExt;

    let mut imap = with_timeout(
        conn::connect(
            &account.imap_host,
            account.imap_port,
            &account.email,
            &account.password,
        ),
        "IMAP connect",
    )
    .await
    .context("Failed to connect to IMAP")?;

    let session = &mut imap.session;

    // Build Sent candidates
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(list_stream) = with_timeout(session.list(None, Some("*")), "IMAP LIST").await {
        let mut names = Vec::new();
        let mut s = list_stream;
        while let Some(item) = s.next().await {
            if let Ok(m) = item {
                names.push(m.name().to_string());
            }
        }
        candidates = crate::imap::folders::detect_sent_candidates(&names);
    }
    if candidates.is_empty() {
        candidates = vec![
            "[Gmail]/Sent Mail".into(),
            "Sent".into(),
            "Sent Items".into(),
            "Sent Messages".into(),
            "INBOX.Sent".into(),
        ];
    }
    if account.provider.as_str() == "gmail" {
        if let Some(pos) = candidates.iter().position(|f| f == "[Gmail]/Sent Mail") {
            if pos != 0 {
                let f = candidates.remove(pos);
                candidates.insert(0, f);
            }
        }
        // Also consider All Mail for Gmail, sometimes visible before Sent label indexing
        if !candidates.iter().any(|f| f == "[Gmail]/All Mail") {
            candidates.insert(1.min(candidates.len()), "[Gmail]/All Mail".into());
        }
    }

    let target = message_id.trim_matches(['<', '>']);

    // Build search variants (prefer Gmail raw search when provider is gmail)
    let header_variants = vec![
        format!("HEADER Message-ID \"{}\"", target),
        format!("HEADER Message-ID \"<{}>\"", target),
        format!("HEADER Message-Id \"{}\"", target),
        format!("HEADER Message-Id \"<{}>\"", target),
    ];
    let gmail_variants = vec![
        format!("X-GM-RAW \"rfc822msgid:{}\"", target),
        format!("X-GM-RAW \"rfc822msgid:\\<{}\\>\"", target),
    ];

    for folder in candidates {
        if with_timeout(session.select(&folder), "IMAP SELECT").await.is_err() {
            continue;
        }

        // 1) Direct UID SEARCH by Message-Id (fast path)
        // Gmail raw search first for gmail accounts
        if account.provider.as_str() == "gmail" {
            for q in &gmail_variants {
                if let Ok(uids) = with_timeout(session.uid_search(q), "IMAP UID SEARCH").await {
                    if let Some(uid) = uids.iter().copied().max() {
                        let _ = with_timeout(session.uid_store(&uid.to_string(), "+FLAGS (\\Seen)"), "IMAP UID STORE").await;
                        upsert_sent_message(pool, account, &folder, uid, subject, to).await?;
                        let _ = session.logout().await;
                        return Ok(Some((folder, uid)));
                    }
                }
            }
        }
        // Standard header variants
        for q in &header_variants {
            if let Ok(uids) = with_timeout(session.uid_search(q), "IMAP UID SEARCH").await {
                if let Some(uid) = uids.iter().copied().max() {
                    let _ = with_timeout(session.uid_store(&uid.to_string(), "+FLAGS (\\Seen)"), "IMAP UID STORE").await;
                    upsert_sent_message(pool, account, &folder, uid, subject, to).await?;
                    let _ = session.logout().await;
                    return Ok(Some((folder, uid)));
                }
            }
        }

        // 2) Fallback: quick scan of recent messages in folder
        let uids_res = match with_timeout(session.uid_search("ALL"), "IMAP UID SEARCH").await {
            Ok(v) => v,
            Err(_) => continue,
        };
        if uids_res.is_empty() {
            continue;
        }
        let mut uid_vec: Vec<u32> = uids_res.into_iter().collect();
        uid_vec.sort_unstable();
        let take = if max_scan == 0 { 100 } else { max_scan.min(uid_vec.len()) };
        let recent: Vec<u32> = uid_vec.iter().rev().take(take).copied().collect();
        let mut recent_sorted = recent.clone();
        recent_sorted.sort_unstable();

        // Fetch in chunks
        for chunk in recent_sorted.chunks(50) {
            let uid_set = chunk
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let fetches = match with_timeout(
                session.uid_fetch(&uid_set, "(UID ENVELOPE BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])"),
                "IMAP UID FETCH",
            )
            .await
            {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Process fetch stream in its own scope to drop the borrow before reusing `session`
            let mut found: Option<u32> = None;
            {
                use futures::StreamExt;
                let mut stream = fetches;
                while let Some(item) = stream.next().await {
                    if let Ok(f) = item {
                        let uid = match f.uid { Some(u) => u, None => continue };
                        let env_mid = f
                            .envelope()
                            .and_then(|e| e.message_id.as_ref())
                            .and_then(|id| std::str::from_utf8(id).ok())
                            .map(|s| s.trim_matches(['<', '>']).to_string());
                        if let Some(mid) = env_mid {
                            if mid == target {
                                found = Some(uid);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(uid) = found {
                let _ = with_timeout(session.uid_store(&uid.to_string(), "+FLAGS (\\Seen)"), "IMAP UID STORE").await;
                upsert_sent_message(pool, account, &folder, uid, subject, to).await?;
                let _ = session.logout().await;
                return Ok(Some((folder, uid)));
            }
        }
    }

    let _ = session.logout().await;
    Ok(None)
}

/// Backfill attachment metadata for existing messages imported before attachments persistence was added.
pub async fn backfill_attachments(
    pool: &SqlitePool,
    account: &Account,
    folder_opt: Option<&str>,
    limit_per_folder: usize,
) -> Result<BackfillStats> {
    use futures::StreamExt;
    // Connect once
    let mut imap_session = with_timeout(
        conn::connect(
            &account.imap_host,
            account.imap_port,
            &account.email,
            &account.password,
        ),
        "IMAP connect",
    )
    .await?;
    let session = &mut imap_session.session;

    // Determine folders
    let folder_list: Vec<String> = if let Some(f) = folder_opt {
        vec![f.to_string()]
    } else {
        let mut names = Vec::new();
        if let Ok(list_stream) = with_timeout(session.list(None, Some("*")), "IMAP LIST").await {
            let mut s = list_stream;
            while let Some(item) = s.next().await {
                if let Ok(m) = item {
                    names.push(m.name().to_string());
                }
            }
        }
        names
    };

    let mut total_scanned = 0usize;
    let mut total_updated = 0usize;
    let mut folder_stats = Vec::new();

    for folder in folder_list.iter() {
        // Collect candidate messages needing backfill (has_attachments=0)
        let rows = sqlx::query(
            "SELECT id, uid FROM messages WHERE account_id = ? AND folder = ? AND has_attachments = 0 ORDER BY uid DESC LIMIT ?",
        )
        .bind(&account.id)
        .bind(folder)
        .bind(limit_per_folder as i64)
        .fetch_all(pool)
        .await?;
        if rows.is_empty() {
            continue;
        }
        let mut id_uid_pairs: Vec<(i64, u32)> = Vec::new();
        for r in &rows {
            let id: i64 = r.try_get(0)?;
            let uid: i64 = r.try_get(1)?;
            id_uid_pairs.push((id, uid as u32));
        }

        let mut scanned = 0usize;
        let mut updated = 0usize;

        // Select folder once
        if with_timeout(session.select(folder), "IMAP SELECT").await.is_err() {
            continue;
        }

        // Chunk UIDs
        const CHUNK: usize = 40;
        for chunk in id_uid_pairs.chunks(CHUNK) {
            let uid_set = chunk
                .iter()
                .map(|(_, u)| u.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let fetches = match with_timeout(
                session.uid_fetch(&uid_set, "(UID BODY.PEEK[])"),
                "UID FETCH",
            )
            .await
            {
                Ok(s) => s,
                Err(_) => continue,
            };

            let mut stream = fetches;
            while let Some(item) = stream.next().await {
                if let Ok(f) = item {
                    if let Some(uid) = f.uid {
                        scanned += 1;
                        total_scanned += 1;
                        if let Some(body) = f.body() {
                            let atts = extract_attachments_from_raw(body);
                            if !atts.is_empty() {
                                // persist
                                if let Some((msg_id, _)) =
                                    id_uid_pairs.iter().find(|(_, u)| *u == uid)
                                {
                                    // delete any existing attachments (rare)
                                    let _ = sqlx::query(
                                        "DELETE FROM attachments WHERE message_id = ?",
                                    )
                                    .bind(msg_id)
                                    .execute(pool)
                                    .await;

                                    for (filename, content_type, sz, content_id, is_inline) in
                                        atts.into_iter()
                                    {
                                        let _ = sqlx::query("INSERT INTO attachments (message_id, filename, content_type, size, content_id, is_inline) VALUES (?, ?, ?, ?, ?, ?)")
                                            .bind(msg_id)
                                            .bind(filename)
                                            .bind(content_type)
                                            .bind(sz)
                                            .bind(content_id)
                                            .bind(is_inline)
                                            .execute(pool)
                                            .await;
                                    }

                                    let _ = sqlx::query(
                                        "UPDATE messages SET has_attachments = 1 WHERE id = ?",
                                    )
                                    .bind(msg_id)
                                    .execute(pool)
                                    .await;

                                    updated += 1;
                                    total_updated += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        folder_stats.push(BackfillFolderStats {
            folder: folder.clone(),
            scanned,
            updated,
        });
    }
    // Logout
    let _ = session.logout().await;

    Ok(BackfillStats {
        account_id: account.id.clone(),
        total_scanned,
        total_updated,
        folders: folder_stats,
    })
}

#[allow(dead_code)]
pub async fn update_last_sync_placeholder() {}
