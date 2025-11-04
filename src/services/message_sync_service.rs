use anyhow::{Context, Result};
use async_imap::types::{Fetch, Flag};
use sqlx::SqlitePool;
use std::collections::HashSet;
use tracing::{info, warn};

use crate::imap::conn;
use crate::models::account::Account;

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
    let mut imap_session = conn::connect(
        &account.imap_host,
        account.imap_port,
        &account.email,
        &account.password,
    )
    .await
    .context("Failed to connect to IMAP")?;

    let session = &mut imap_session.session;

    // Select folder
    let mailbox = session
        .select(folder)
        .await
        .context(format!("Failed to select folder {}", folder))?;

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
    let messages = session
        .fetch(&sequence, "UID")
        .await
        .context("Failed to fetch UIDs")?;

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

            let messages = session
                .uid_fetch(
                    &uid_set,
                    "(UID FLAGS ENVELOPE BODY.PEEK[HEADER] INTERNALDATE RFC822.SIZE)",
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
    let subject = envelope
        .and_then(|e| e.subject.as_ref())
        .and_then(|s| std::str::from_utf8(s).ok())
        .unwrap_or("");

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

    // Try to get message size from fetch data (size field returns Option<u32>)
    let size = fetch.size.unwrap_or(0) as i64;

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
        // Insert new message (without body for now - will be fetched on demand)
        sqlx::query(
            r#"
            INSERT INTO messages (
                account_id, folder, uid, message_id,
                subject, from_addr, to_addr, date,
                flags, size, has_attachments,
                synced_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, datetime('now'))
            "#,
        )
        .bind(&account.id)
        .bind(folder)
        .bind(uid)
        .bind(message_id)
        .bind(subject)
        .bind(&from)
        .bind(&to)
        .bind(date)
        .bind(&flags_json)
        .bind(size as i64)
        .execute(pool)
        .await?;

        Ok(true) // New
    }
}

/// Sync all folders for an account
pub async fn sync_account_messages(pool: &SqlitePool, account: &Account) -> Result<Vec<SyncStats>> {
    let mut imap_session = conn::connect(
        &account.imap_host,
        account.imap_port,
        &account.email,
        &account.password,
    )
    .await?;

    let session = &mut imap_session.session;

    // List all folders
    let folders = session.list(None, Some("*")).await?;

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
