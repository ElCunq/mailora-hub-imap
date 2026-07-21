use anyhow::{anyhow, Result};
use sqlx::{FromRow, SqlitePool};
use serde::{Deserialize, Serialize};
use crate::mailboxes::model::MailboxSyncState;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct JobLock {
    pub lock_key: String,
    pub locked_by: String,
    pub locked_until: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct SyncRunLog {
    pub id: i64,
    pub mailbox_id: i64,
    pub folder_name: String,
    pub sync_type: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub new_count: Option<i64>,
    pub updated_count: Option<i64>,
    pub deleted_count: Option<i64>,
    pub error_count: Option<i64>,
    pub error_message: Option<String>,
}

pub struct SyncStateRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> SyncStateRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_sync_state(&self, mailbox_id: i64, folder_name: &str) -> Result<Option<MailboxSyncState>> {
        let state = sqlx::query_as::<_, MailboxSyncState>(
            "SELECT mailbox_id, folder_name, folder_kind, uid_validity, highest_uid, highest_modseq, last_incremental_sync_at, last_reconciliation_at, last_success_at, last_error, sync_status FROM mailbox_sync_states WHERE mailbox_id = ? AND folder_name = ?"
        )
        .bind(mailbox_id)
        .bind(folder_name)
        .fetch_optional(self.pool)
        .await?;

        Ok(state)
    }

    pub async fn upsert_sync_state(&self, state: &MailboxSyncState) -> Result<MailboxSyncState> {
        sqlx::query(
            r#"INSERT INTO mailbox_sync_states (
                   mailbox_id, folder_name, folder_kind, uid_validity, highest_uid, highest_modseq,
                   last_incremental_sync_at, last_reconciliation_at, last_success_at, last_error, sync_status
               ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(mailbox_id, folder_name) DO UPDATE SET
                   folder_kind = excluded.folder_kind,
                   uid_validity = excluded.uid_validity,
                   highest_uid = excluded.highest_uid,
                   highest_modseq = excluded.highest_modseq,
                   last_incremental_sync_at = COALESCE(excluded.last_incremental_sync_at, mailbox_sync_states.last_incremental_sync_at),
                   last_reconciliation_at = COALESCE(excluded.last_reconciliation_at, mailbox_sync_states.last_reconciliation_at),
                   last_success_at = COALESCE(excluded.last_success_at, mailbox_sync_states.last_success_at),
                   last_error = excluded.last_error,
                   sync_status = excluded.sync_status"#
        )
        .bind(state.mailbox_id)
        .bind(&state.folder_name)
        .bind(&state.folder_kind)
        .bind(state.uid_validity)
        .bind(state.highest_uid)
        .bind(state.highest_modseq)
        .bind(&state.last_incremental_sync_at)
        .bind(&state.last_reconciliation_at)
        .bind(&state.last_success_at)
        .bind(&state.last_error)
        .bind(&state.sync_status)
        .execute(self.pool)
        .await?;

        match self.get_sync_state(state.mailbox_id, &state.folder_name).await? {
            Some(s) => Ok(s),
            None => Err(anyhow!("Failed to retrieve sync state after upsert")),
        }
    }

    pub async fn update_status(&self, mailbox_id: i64, folder_name: &str, status: &str, error: Option<&str>) -> Result<()> {
        let err_bind = error.map(|e| e.to_string());
        sqlx::query(
            "UPDATE mailbox_sync_states SET sync_status = ?, last_error = ? WHERE mailbox_id = ? AND folder_name = ?"
        )
        .bind(status)
        .bind(err_bind)
        .bind(mailbox_id)
        .bind(folder_name)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_success(&self, mailbox_id: i64, folder_name: &str, highest_uid: i64, highest_modseq: Option<i64>, is_reconciliation: bool) -> Result<()> {
        if is_reconciliation {
            sqlx::query(
                "UPDATE mailbox_sync_states SET highest_uid = ?, highest_modseq = ?, last_reconciliation_at = datetime('now'), last_success_at = datetime('now'), sync_status = 'idle', last_error = NULL WHERE mailbox_id = ? AND folder_name = ?"
            )
            .bind(highest_uid)
            .bind(highest_modseq)
            .bind(mailbox_id)
            .bind(folder_name)
            .execute(self.pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE mailbox_sync_states SET highest_uid = ?, highest_modseq = ?, last_incremental_sync_at = datetime('now'), last_success_at = datetime('now'), sync_status = 'idle', last_error = NULL WHERE mailbox_id = ? AND folder_name = ?"
            )
            .bind(highest_uid)
            .bind(highest_modseq)
            .bind(mailbox_id)
            .bind(folder_name)
            .execute(self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn handle_uidvalidity_reset(&self, mailbox_id: i64, folder_name: &str, new_uid_validity: i64) -> Result<()> {
        tracing::warn!(mailbox_id, folder_name, new_uid_validity, "UIDVALIDITY changed! Resetting highest_uid to 0 and deleting cached messages for resync.");
        
        let account_id_str = mailbox_id.to_string();
        sqlx::query("DELETE FROM messages WHERE account_id = ? AND folder = ?")
            .bind(&account_id_str)
            .bind(folder_name)
            .execute(self.pool)
            .await?;

        sqlx::query(
            "UPDATE mailbox_sync_states SET uid_validity = ?, highest_uid = 0, highest_modseq = NULL, sync_status = 'idle', last_error = 'UIDVALIDITY reset' WHERE mailbox_id = ? AND folder_name = ?"
        )
        .bind(new_uid_validity)
        .bind(mailbox_id)
        .bind(folder_name)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_by_mailbox(&self, mailbox_id: i64) -> Result<Vec<MailboxSyncState>> {
        let states = sqlx::query_as::<_, MailboxSyncState>(
            "SELECT mailbox_id, folder_name, folder_kind, uid_validity, highest_uid, highest_modseq, last_incremental_sync_at, last_reconciliation_at, last_success_at, last_error, sync_status FROM mailbox_sync_states WHERE mailbox_id = ? ORDER BY folder_name ASC"
        )
        .bind(mailbox_id)
        .fetch_all(self.pool)
        .await?;

        Ok(states)
    }
}

pub struct JobLockRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> JobLockRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Try to acquire a job lock. Returns true if acquired, false if already locked by someone else.
    pub async fn acquire_lock(&self, lock_key: &str, locked_by: &str, duration_secs: i64) -> Result<bool> {
        // Clean up expired locks first
        let _ = sqlx::query("DELETE FROM job_locks WHERE locked_until < datetime('now')")
            .execute(self.pool)
            .await;

        let res = sqlx::query(
            r#"INSERT INTO job_locks (lock_key, locked_by, locked_until, created_at)
               VALUES (?, ?, datetime('now', ? || ' seconds'), datetime('now'))"#
        )
        .bind(lock_key)
        .bind(locked_by)
        .bind(duration_secs.to_string())
        .execute(self.pool)
        .await;

        match res {
            Ok(_) => Ok(true),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("UNIQUE constraint") || msg.contains("already exists") || msg.contains("not unique") {
                    Ok(false)
                } else {
                    Err(anyhow!(e))
                }
            }
        }
    }

    pub async fn release_lock(&self, lock_key: &str, locked_by: &str) -> Result<()> {
        sqlx::query("DELETE FROM job_locks WHERE lock_key = ? AND locked_by = ?")
            .bind(lock_key)
            .bind(locked_by)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn extend_lock(&self, lock_key: &str, locked_by: &str, duration_secs: i64) -> Result<bool> {
        let res = sqlx::query(
            "UPDATE job_locks SET locked_until = datetime('now', ? || ' seconds') WHERE lock_key = ? AND locked_by = ?"
        )
        .bind(duration_secs.to_string())
        .bind(lock_key)
        .bind(locked_by)
        .execute(self.pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }
}

pub struct SyncRunRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> SyncRunRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn start_run(&self, mailbox_id: i64, folder_name: &str, sync_type: &str) -> Result<i64> {
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO sync_runs (mailbox_id, folder_name, sync_type, status, started_at)
               VALUES (?, ?, ?, 'running', datetime('now'))
               RETURNING id"#
        )
        .bind(mailbox_id)
        .bind(folder_name)
        .bind(sync_type)
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn finish_run(
        &self,
        run_id: i64,
        status: &str,
        new_count: i64,
        updated_count: i64,
        deleted_count: i64,
        error_count: i64,
        error_message: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE sync_runs
               SET status = ?, finished_at = datetime('now'),
                   duration_ms = CAST((julianday(datetime('now')) - julianday(started_at)) * 86400000 AS INTEGER),
                   new_count = ?, updated_count = ?, deleted_count = ?, error_count = ?, error_message = ?
               WHERE id = ?"#
        )
        .bind(status)
        .bind(new_count)
        .bind(updated_count)
        .bind(deleted_count)
        .bind(error_count)
        .bind(error_message)
        .bind(run_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
