use anyhow::{anyhow, Context, Result};
use async_imap::types::Flag;
use futures::StreamExt;
use mail_parser::MimeHeaders;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use crate::domains::DomainRepository;
use crate::imap::conn;
use crate::mailboxes::{
    model::MailboxSyncState, JobLockRepository, MailboxRepository, SyncRunRepository, SyncStateRepository,
};
use crate::mailcow::MailcowRepository;
use crate::services::crypto::decrypt_secret;

#[derive(Debug, Clone, serde::Serialize)]
pub struct MailboxSyncSummary {
    pub mailbox_id: i64,
    pub address: String,
    pub folders_synced: usize,
    pub total_new_messages: usize,
    pub total_updated_messages: usize,
    pub total_deleted_messages: usize,
    pub duration_ms: u64,
    pub skipped_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FolderSyncSummary {
    pub folder_name: String,
    pub new_messages: usize,
    pub updated_messages: usize,
    pub deleted_messages: usize,
}

pub struct SyncCoordinator {
    pub pool: SqlitePool,
    pub semaphore: Arc<Semaphore>,
}

impl SyncCoordinator {
    pub fn new(pool: SqlitePool, max_concurrency: usize) -> Self {
        Self {
            pool,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    /// Helper to determine standardized folder kind from folder name.
    pub fn derive_folder_kind(name: &str) -> Option<String> {
        let lower = name.to_lowercase();
        if lower == "inbox" {
            Some("inbox".to_string())
        } else if lower == "sent" || lower.contains("sent mail") || lower.contains("sent items") {
            Some("sent".to_string())
        } else if lower == "drafts" {
            Some("drafts".to_string())
        } else if lower == "trash" || lower.contains("deleted") || lower == "bin" {
            Some("trash".to_string())
        } else if lower == "junk" || lower == "spam" {
            Some("junk".to_string())
        } else if lower == "archive" || lower == "archives" {
            Some("archive".to_string())
        } else {
            None
        }
    }

    /// Main synchronization entry point for a single mailbox.
    pub async fn sync_mailbox(&self, mailbox_id: i64, is_reconciliation: bool) -> Result<MailboxSyncSummary> {
        let start = std::time::Instant::now();

        // 1. Acquire bounded process concurrency permit
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| anyhow!("Failed to acquire concurrency permit: {}", e))?;

        // 2. Acquire database job lock (preventing multiple workers/nodes from syncing the same mailbox)
        let job_locks = JobLockRepository::new(&self.pool);
        let lock_key = format!("mailbox_sync:{}", mailbox_id);
        let acquired = job_locks
            .acquire_lock(&lock_key, "sync_coordinator", 300)
            .await?;

        if !acquired {
            warn!(mailbox_id, "Sync job already locked and running for this mailbox. Skipping.");
            return Ok(MailboxSyncSummary {
                mailbox_id,
                address: String::new(),
                folders_synced: 0,
                total_new_messages: 0,
                total_updated_messages: 0,
                total_deleted_messages: 0,
                duration_ms: start.elapsed().as_millis() as u64,
                skipped_reason: Some("Job already locked/running".to_string()),
            });
        }

        let sync_runs = SyncRunRepository::new(&self.pool);
        let run_id = sync_runs
            .start_run(
                mailbox_id,
                "ALL",
                if is_reconciliation { "reconciliation" } else { "incremental" },
            )
            .await?;

        // Ensure lock release and run tracking closure
        let result = self.sync_mailbox_internal(mailbox_id, is_reconciliation).await;

        match &result {
            Ok(summary) => {
                let _ = sync_runs
                    .finish_run(
                        run_id,
                        "success",
                        summary.total_new_messages as i64,
                        summary.total_updated_messages as i64,
                        summary.total_deleted_messages as i64,
                        0,
                        None,
                    )
                    .await;
            }
            Err(err) => {
                let err_msg = err.to_string();
                error!(mailbox_id, error = %err_msg, "Mailbox sync failed");
                let _ = sync_runs
                    .finish_run(run_id, "error", 0, 0, 0, 1, Some(&err_msg))
                    .await;
            }
        }

        // Always release job lock
        let _ = job_locks.release_lock(&lock_key, "sync_coordinator").await;

        result
    }

    async fn sync_mailbox_internal(&self, mailbox_id: i64, is_reconciliation: bool) -> Result<MailboxSyncSummary> {
        let start = std::time::Instant::now();
        let mb_repo = MailboxRepository::new(&self.pool);
        let mailbox = mb_repo.get_by_id(mailbox_id).await.context("Mailbox not found")?;

        if !mailbox.active || mailbox.deleted_at.is_some() {
            return Ok(MailboxSyncSummary {
                mailbox_id,
                address: mailbox.address,
                folders_synced: 0,
                total_new_messages: 0,
                total_updated_messages: 0,
                total_deleted_messages: 0,
                duration_ms: start.elapsed().as_millis() as u64,
                skipped_reason: Some("Mailbox inactive or deleted".to_string()),
            });
        }

        let cred = mb_repo
            .get_credentials(mailbox_id)
            .await
            .context("Mailbox credentials missing")?;
        let decrypted_pass = decrypt_secret(&cred.password_encrypted)
            .context("Failed to decrypt password")?;

        let domain_repo = DomainRepository::new(&self.pool);
        let domain = domain_repo
            .get_by_id(mailbox.domain_id)
            .await
            .context("Domain not found")?;

        let mailcow_repo = MailcowRepository::new(&self.pool);
        let instance = mailcow_repo
            .get_by_id(domain.mailcow_instance_id)
            .await
            .context("Mailcow instance not found")?;

        info!(mailbox_id, address = %mailbox.address, host = %instance.imap_host, "Connecting to IMAP server");

        let mut imap_session = conn::connect(
            &instance.imap_host,
            instance.imap_port as u16,
            &cred.username,
            &decrypted_pass,
        )
        .await
        .context("IMAP connect failed")?;

        let session = &mut imap_session.session;

        // Discover folders
        let mut folders = Vec::new();
        if let Ok(mut list_stream) = session.list(None, Some("*")).await {
            while let Some(item) = list_stream.next().await {
                if let Ok(f) = item {
                    folders.push(f.name().to_string());
                }
            }
        }
        if folders.is_empty() {
            folders.push("INBOX".to_string());
        }

        let sync_states = SyncStateRepository::new(&self.pool);
        let mut total_new = 0;
        let mut total_updated = 0;
        let mut total_deleted = 0;
        let mut folders_synced = 0;

        for folder in &folders {
            match self
                .sync_folder(
                    session,
                    mailbox_id,
                    &mailbox.address,
                    folder,
                    is_reconciliation,
                    &sync_states,
                )
                .await
            {
                Ok(summary) => {
                    total_new += summary.new_messages;
                    total_updated += summary.updated_messages;
                    total_deleted += summary.deleted_messages;
                    folders_synced += 1;
                }
                Err(e) => {
                    error!(mailbox_id, folder, error = %e, "Folder sync error");
                    let _ = sync_states
                        .update_status(mailbox_id, folder, "error", Some(&e.to_string()))
                        .await;
                }
            }
        }

        // Trigger deduplicated prefetch of pending bodies and attachments across all folders synced
        let _ = self
            .prefetch_pending_bodies_and_attachments(session, mailbox_id, 100)
            .await;

        let _ = session.logout().await;

        Ok(MailboxSyncSummary {
            mailbox_id,
            address: mailbox.address,
            folders_synced,
            total_new_messages: total_new,
            total_updated_messages: total_updated,
            total_deleted_messages: total_deleted,
            duration_ms: start.elapsed().as_millis() as u64,
            skipped_reason: None,
        })
    }

    async fn sync_folder(
        &self,
        session: &mut async_imap::Session<tokio_util::compat::Compat<tokio_native_tls::TlsStream<tokio::net::TcpStream>>>,
        mailbox_id: i64,
        _address: &str,
        folder_name: &str,
        is_reconciliation: bool,
        sync_states: &SyncStateRepository<'_>,
    ) -> Result<FolderSyncSummary> {
        let mb_select = session
            .select(folder_name)
            .await
            .context("IMAP SELECT folder failed")?;

        let server_uid_validity = mb_select.uid_validity.unwrap_or(0) as i64;
        let current_state = sync_states.get_sync_state(mailbox_id, folder_name).await?;

        // Check UIDVALIDITY reset condition
        if let Some(ref st) = current_state {
            if st.uid_validity != server_uid_validity && st.uid_validity != 0 && server_uid_validity != 0 {
                sync_states
                    .handle_uidvalidity_reset(mailbox_id, folder_name, server_uid_validity)
                    .await?;
            }
        }

        let state = match sync_states.get_sync_state(mailbox_id, folder_name).await? {
            Some(s) => s,
            None => {
                let init = MailboxSyncState {
                    mailbox_id,
                    folder_name: folder_name.to_string(),
                    folder_kind: Self::derive_folder_kind(folder_name),
                    uid_validity: server_uid_validity,
                    highest_uid: 0,
                    highest_modseq: mb_select.highest_modseq.map(|v| v as i64),
                    last_incremental_sync_at: None,
                    last_reconciliation_at: None,
                    last_success_at: None,
                    last_error: None,
                    sync_status: "syncing".to_string(),
                };
                sync_states.upsert_sync_state(&init).await?
            }
        };

        sync_states
            .update_status(mailbox_id, folder_name, "syncing", None)
            .await?;

        let highest_uid = state.highest_uid as u32;

        // Fetch server UIDs
        let all_uids_vec = session.uid_search("ALL").await.unwrap_or_default();
        let server_uids: HashSet<u32> = all_uids_vec.iter().copied().collect();

        let mut new_uids: Vec<u32> = all_uids_vec
            .into_iter()
            .filter(|&u| u > highest_uid)
            .collect();
        new_uids.sort_unstable_by(|a, b| b.cmp(a)); // Newest first

        // Bounded batch sync cap
        if new_uids.len() > 1000 {
            info!(mailbox_id, folder_name, count = new_uids.len(), "Capping new message fetch batch to 1000");
            new_uids.truncate(1000);
        }

        let account_id_str = mailbox_id.to_string();
        let mut new_messages_count = 0;
        let mut updated_messages_count = 0;

        // Fetch envelope/headers and flags in chunks
        const CHUNK_SIZE: usize = 50;
        for chunk in new_uids.chunks(CHUNK_SIZE) {
            let seq = chunk
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let fetches = session
                .uid_fetch(
                    &seq,
                    "(UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE BODY.PEEK[HEADER.FIELDS (MESSAGE-ID REFERENCES IN-REPLY-TO CONTENT-TYPE)])",
                )
                .await?;

            let mut stream = fetches;
            while let Some(item) = stream.next().await {
                if let Ok(fetch) = item {
                    if let Some(uid) = fetch.uid {
                        let header_bytes = fetch.header().unwrap_or(b"");
                        let mut subject = String::new();
                        let mut from = String::new();
                        let mut to = String::new();
                        let mut date = String::new();
                        let mut message_id = String::new();

                        if !header_bytes.is_empty() {
                            if let Some(parsed) = mail_parser::Message::parse(header_bytes) {
                                subject = parsed.subject().unwrap_or("").to_string();
                                match parsed.from() {
                                    mail_parser::HeaderValue::Address(addr) => {
                                        let name = addr.name.as_ref().map(|n| n.as_ref()).unwrap_or("");
                                        let email = addr.address.as_ref().map(|a| a.as_ref()).unwrap_or("");
                                        from = if !name.is_empty() {
                                            format!("{} <{}>", name, email)
                                        } else {
                                            email.to_string()
                                        };
                                    }
                                    mail_parser::HeaderValue::AddressList(list) => {
                                        if let Some(addr) = list.first() {
                                            let name = addr.name.as_ref().map(|n| n.as_ref()).unwrap_or("");
                                            let email = addr.address.as_ref().map(|a| a.as_ref()).unwrap_or("");
                                            from = if !name.is_empty() {
                                                format!("{} <{}>", name, email)
                                            } else {
                                                email.to_string()
                                            };
                                        }
                                    }
                                    _ => {}
                                }
                                match parsed.to() {
                                    mail_parser::HeaderValue::Address(addr) => {
                                        to = addr.address.as_ref().map(|a| a.as_ref()).unwrap_or("").to_string();
                                    }
                                    mail_parser::HeaderValue::AddressList(list) => {
                                        if let Some(addr) = list.first() {
                                            to = addr.address.as_ref().map(|a| a.as_ref()).unwrap_or("").to_string();
                                        }
                                    }
                                    _ => {}
                                }
                                if let Some(d) = parsed.date() {
                                    date = d.to_rfc3339();
                                }
                                message_id = parsed.message_id().unwrap_or("").to_string();
                            }
                        }

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
                        let flags_json = serde_json::to_string(&flags).unwrap_or_else(|_| "[]".to_string());
                        let size = fetch.size.unwrap_or(0) as i64;
                        let has_attachments = false; // Will be accurately set during body/attachment prefetch

                        let is_seen = flags.contains(&"\\Seen".to_string());
                        let is_answered = flags.contains(&"\\Answered".to_string());
                        let is_flagged = flags.contains(&"\\Flagged".to_string());
                        let is_deleted = flags.contains(&"\\Deleted".to_string());
                        let is_draft = flags.contains(&"\\Draft".to_string());
                        let folder_kind = Self::derive_folder_kind(folder_name);
                        let internal_date_ts = fetch
                            .internal_date()
                            .map(|dt| dt.timestamp())
                            .or_else(|| chrono::DateTime::parse_from_rfc3339(&date).map(|dt| dt.timestamp()).ok())
                            .unwrap_or(0);

                        sqlx::query(
                            r#"INSERT INTO messages (
                                   account_id, folder, uid, message_id, subject, from_addr, to_addr, date,
                                   flags, is_seen, is_answered, is_flagged, is_deleted, is_draft, folder_kind, internal_date_ts,
                                   size, has_attachments, synced_at
                               ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
                               ON CONFLICT (account_id, folder, uid) DO UPDATE SET
                                   message_id = COALESCE(NULLIF(excluded.message_id, ''), messages.message_id),
                                   subject = COALESCE(NULLIF(excluded.subject, ''), messages.subject),
                                   from_addr = COALESCE(NULLIF(excluded.from_addr, ''), messages.from_addr),
                                   to_addr = COALESCE(NULLIF(excluded.to_addr, ''), messages.to_addr),
                                   date = COALESCE(NULLIF(excluded.date, ''), messages.date),
                                   flags = excluded.flags,
                                   is_seen = excluded.is_seen,
                                   is_answered = excluded.is_answered,
                                   is_flagged = excluded.is_flagged,
                                   is_deleted = excluded.is_deleted,
                                   is_draft = excluded.is_draft,
                                   folder_kind = COALESCE(excluded.folder_kind, messages.folder_kind),
                                   internal_date_ts = CASE WHEN excluded.internal_date_ts > 0 THEN excluded.internal_date_ts ELSE messages.internal_date_ts END,
                                   size = CASE WHEN excluded.size > 0 THEN excluded.size ELSE messages.size END,
                                   synced_at = datetime('now')"#
                        )
                        .bind(&account_id_str)
                        .bind(folder_name)
                        .bind(uid as i64)
                        .bind(&message_id)
                        .bind(&subject)
                        .bind(&from)
                        .bind(&to)
                        .bind(&date)
                        .bind(&flags_json)
                        .bind(is_seen)
                        .bind(is_answered)
                        .bind(is_flagged)
                        .bind(is_deleted)
                        .bind(is_draft)
                        .bind(&folder_kind)
                        .bind(internal_date_ts)
                        .bind(size)
                        .bind(has_attachments)
                        .execute(&self.pool)
                        .await?;

                        if uid > highest_uid {
                            new_messages_count += 1;
                        } else {
                            updated_messages_count += 1;
                        }
                    }
                }
            }
        }

        // Reconciliation: Detect deleted messages and check flags for existing messages if requested
        let mut deleted_messages_count = 0;
        if is_reconciliation {
            let existing_uids: HashSet<u32> = sqlx::query_scalar(
                "SELECT uid FROM messages WHERE account_id = ? AND folder = ?",
            )
            .bind(&account_id_str)
            .bind(folder_name)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|u: i64| u as u32)
            .collect();

            let deleted_uids: Vec<u32> = existing_uids
                .difference(&server_uids)
                .copied()
                .collect();

            for chunk in deleted_uids.chunks(100) {
                let place_holders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sql = format!(
                    "DELETE FROM messages WHERE account_id = ? AND folder = ? AND uid IN ({})",
                    place_holders
                );
                let mut q = sqlx::query(&sql)
                    .bind(&account_id_str)
                    .bind(folder_name);
                for &u in chunk {
                    q = q.bind(u as i64);
                }
                if let Ok(res) = q.execute(&self.pool).await {
                    deleted_messages_count += res.rows_affected() as usize;
                }
            }
        }

        let max_synced_uid = new_uids
            .iter()
            .max()
            .copied()
            .unwrap_or(highest_uid)
            .max(highest_uid) as i64;

        sync_states
            .update_success(
                mailbox_id,
                folder_name,
                max_synced_uid,
                mb_select.highest_modseq.map(|v| v as i64),
                is_reconciliation,
            )
            .await?;

        Ok(FolderSyncSummary {
            folder_name: folder_name.to_string(),
            new_messages: new_messages_count,
            updated_messages: updated_messages_count,
            deleted_messages: deleted_messages_count,
        })
    }

    /// Deduplicated body and attachment prefetch queue processing
    pub async fn prefetch_pending_bodies_and_attachments(
        &self,
        session: &mut async_imap::Session<tokio_util::compat::Compat<tokio_native_tls::TlsStream<tokio::net::TcpStream>>>,
        mailbox_id: i64,
        limit: usize,
    ) -> Result<()> {
        let account_id_str = mailbox_id.to_string();
        let pending = sqlx::query(
            r#"SELECT id, folder, uid FROM messages
               WHERE account_id = ? AND body_plain IS NULL AND body_html IS NULL
               ORDER BY uid DESC LIMIT ?"#
        )
        .bind(&account_id_str)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        if pending.is_empty() {
            return Ok(());
        }

        info!(mailbox_id, count = pending.len(), "Prefetching pending bodies and attachments");

        // Group pending messages by folder so we select each folder only once
        use std::collections::HashMap;
        let mut by_folder: HashMap<String, Vec<(i64, u32)>> = HashMap::new();
        for row in pending {
            use sqlx::Row;
            let msg_id: i64 = row.get("id");
            let folder: String = row.get("folder");
            let uid: i64 = row.get("uid");
            by_folder.entry(folder).or_default().push((msg_id, uid as u32));
        }

        for (folder, items) in by_folder {
            if session.select(&folder).await.is_err() {
                continue;
            }

            const CHUNK_SIZE: usize = 20;
            for chunk in items.chunks(CHUNK_SIZE) {
                let seq = chunk
                    .iter()
                    .map(|(_, u)| u.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                if let Ok(mut fetches) = session.uid_fetch(&seq, "(UID BODY.PEEK[])").await {
                    while let Some(item) = fetches.next().await {
                        if let Ok(fetch) = item {
                            if let Some(uid) = fetch.uid {
                                if let Some((msg_id, _)) = chunk.iter().find(|(_, u)| *u == uid) {
                                    let body_bytes = fetch.body().unwrap_or(b"");
                                    if !body_bytes.is_empty() {
                                        if let Some(parsed) = mail_parser::Message::parse(body_bytes) {
                                            let body_plain = parsed
                                                .body_text(0)
                                                .unwrap_or(std::borrow::Cow::Borrowed(""))
                                                .to_string();
                                            let body_html = parsed.body_html(0).map(|s| s.to_string());

                                            // Save bodies
                                            let _ = sqlx::query(
                                                "UPDATE messages SET body_plain = ?, body_html = ? WHERE id = ?"
                                            )
                                            .bind(&body_plain)
                                            .bind(&body_html)
                                            .bind(msg_id)
                                            .execute(&self.pool)
                                            .await;

                                            // Deduplicated attachment extraction
                                            let mut atts_found = false;
                                            for part in parsed.parts.iter() {
                                                let ctype = part.content_type();
                                                let c_type = ctype.map(|c| c.c_type.as_ref()).unwrap_or("application");
                                                let subtype = ctype.and_then(|c| c.subtype()).unwrap_or("");
                                                if c_type == "multipart" { continue; }
                                                let is_body = c_type == "text" && (subtype == "plain" || subtype == "html");
                                                let has_filename = part.attachment_name().is_some();
                                                let is_media = c_type == "image" || c_type == "video" || c_type == "audio" || c_type == "application";

                                                if has_filename || (is_media && !is_body) {
                                                    let fname = part.attachment_name().map(|s| s.to_string());
                                                    let ctype_full = format!("{}/{}", c_type, subtype);
                                                    let sz = part.contents().len() as i64;
                                                    let cid = part.content_id().map(|s| s.to_string());
                                                    let mut is_inline = false;
                                                    if let Some(cd) = part.content_disposition() {
                                                        if cd.c_type.eq_ignore_ascii_case("inline") { is_inline = true; }
                                                    }
                                                    if cid.is_some() { is_inline = true; }

                                                    atts_found = true;

                                                    // Deduplication check
                                                    let exists_att: bool = sqlx::query_scalar(
                                                        "SELECT COUNT(*) > 0 FROM attachments WHERE message_id = ? AND COALESCE(filename, '') = COALESCE(?, '')"
                                                    )
                                                    .bind(msg_id)
                                                    .bind(&fname)
                                                    .fetch_one(&self.pool)
                                                    .await
                                                    .unwrap_or(false);

                                                    if !exists_att {
                                                        let _ = sqlx::query(
                                                            "INSERT INTO attachments (message_id, filename, content_type, size, content_id, is_inline) VALUES (?, ?, ?, ?, ?, ?)"
                                                        )
                                                        .bind(msg_id)
                                                        .bind(&fname)
                                                        .bind(&ctype_full)
                                                        .bind(sz)
                                                        .bind(&cid)
                                                        .bind(is_inline)
                                                        .execute(&self.pool)
                                                        .await;
                                                    }
                                                }
                                            }

                                            if atts_found {
                                                let _ = sqlx::query("UPDATE messages SET has_attachments = 1 WHERE id = ?")
                                                    .bind(msg_id)
                                                    .execute(&self.pool)
                                                    .await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
