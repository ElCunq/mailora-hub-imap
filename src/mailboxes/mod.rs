pub mod model;
pub mod sync_state;

use anyhow::Result;
use sqlx::SqlitePool;
pub use model::{MailboxAssignment, MailboxCredential, MailcowMailbox, MailcowMailbox as Mailbox, UpsertMailbox};
pub use sync_state::{JobLock, JobLockRepository, SyncRunLog, SyncRunRepository, SyncStateRepository};

pub struct MailboxRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> MailboxRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, req: UpsertMailbox) -> Result<MailcowMailbox> {
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO mailboxes (domain_id, external_id, address, local_part, display_name, active, quota, used_quota, credential_status, connection_status, sync_status, last_seen_at, deleted_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'missing', 'not_tested', 'idle', datetime('now'), NULL, datetime('now'))
               ON CONFLICT(address) DO UPDATE SET
                   domain_id = excluded.domain_id,
                   external_id = excluded.external_id,
                   local_part = excluded.local_part,
                   display_name = excluded.display_name,
                   active = excluded.active,
                   quota = excluded.quota,
                   used_quota = excluded.used_quota,
                   last_seen_at = datetime('now'),
                   deleted_at = NULL,
                   updated_at = datetime('now')
               RETURNING id"#,
        )
        .bind(req.domain_id)
        .bind(&req.external_id)
        .bind(&req.address)
        .bind(&req.local_part)
        .bind(&req.display_name)
        .bind(req.active)
        .bind(req.quota)
        .bind(req.used_quota)
        .fetch_one(self.pool)
        .await?;

        self.get_by_id(id).await
    }

    pub async fn get_by_id(&self, id: i64) -> Result<MailcowMailbox> {
        let mb = sqlx::query_as::<_, MailcowMailbox>(
            "SELECT id, domain_id, external_id, address, local_part, display_name, active, quota, used_quota, credential_status, connection_status, sync_status, last_seen_at, deleted_at, created_at, updated_at FROM mailboxes WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(mb)
    }

    pub async fn get_by_address(&self, address: &str) -> Result<MailcowMailbox> {
        let mb = sqlx::query_as::<_, MailcowMailbox>(
            "SELECT id, domain_id, external_id, address, local_part, display_name, active, quota, used_quota, credential_status, connection_status, sync_status, last_seen_at, deleted_at, created_at, updated_at FROM mailboxes WHERE address = ?",
        )
        .bind(address)
        .fetch_one(self.pool)
        .await?;

        Ok(mb)
    }

    pub async fn list_by_domain(&self, domain_id: i64) -> Result<Vec<MailcowMailbox>> {
        let mbs = sqlx::query_as::<_, MailcowMailbox>(
            "SELECT id, domain_id, external_id, address, local_part, display_name, active, quota, used_quota, credential_status, connection_status, sync_status, last_seen_at, deleted_at, created_at, updated_at FROM mailboxes WHERE domain_id = ? AND deleted_at IS NULL ORDER BY address ASC",
        )
        .bind(domain_id)
        .fetch_all(self.pool)
        .await?;

        Ok(mbs)
    }

    pub async fn soft_delete_unseen(&self, domain_id: i64, seen_before: &str) -> Result<u64> {
        let res = sqlx::query(
            "UPDATE mailboxes SET deleted_at = datetime('now'), active = 0, updated_at = datetime('now') WHERE domain_id = ? AND last_seen_at < ? AND deleted_at IS NULL",
        )
        .bind(domain_id)
        .bind(seen_before)
        .execute(self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn save_credentials(
        &self,
        mailbox_id: i64,
        username: &str,
        password_encrypted: &str,
    ) -> Result<MailboxCredential> {
        sqlx::query(
            r#"INSERT INTO mailbox_credentials (mailbox_id, username, password_encrypted, credential_type, created_at, updated_at)
               VALUES (?, ?, ?, 'password', datetime('now'), datetime('now'))
               ON CONFLICT(mailbox_id) DO UPDATE SET
                   username = excluded.username,
                   password_encrypted = excluded.password_encrypted,
                   updated_at = datetime('now')"#,
        )
        .bind(mailbox_id)
        .bind(username)
        .bind(password_encrypted)
        .execute(self.pool)
        .await?;

        sqlx::query(
            "UPDATE mailboxes SET credential_status = 'present', updated_at = datetime('now') WHERE id = ?",
        )
        .bind(mailbox_id)
        .execute(self.pool)
        .await?;

        let cred = sqlx::query_as::<_, MailboxCredential>(
            "SELECT mailbox_id, username, password_encrypted, credential_type, created_at, updated_at, last_verified_at, verification_status, verification_error FROM mailbox_credentials WHERE mailbox_id = ?",
        )
        .bind(mailbox_id)
        .fetch_one(self.pool)
        .await?;

        Ok(cred)
    }

    pub async fn get_credentials(&self, mailbox_id: i64) -> Result<MailboxCredential> {
        let cred = sqlx::query_as::<_, MailboxCredential>(
            "SELECT mailbox_id, username, password_encrypted, credential_type, created_at, updated_at, last_verified_at, verification_status, verification_error FROM mailbox_credentials WHERE mailbox_id = ?",
        )
        .bind(mailbox_id)
        .fetch_one(self.pool)
        .await?;

        Ok(cred)
    }

    pub async fn assign_user(
        &self,
        user_id: i64,
        mailbox_id: i64,
        permissions: (bool, bool, bool, bool, bool, bool, bool, bool, bool),
        assigned_by: Option<i64>,
    ) -> Result<MailboxAssignment> {
        let (can_view, can_read, can_reply, can_send, can_send_as, can_mark_read, can_move, can_delete, can_manage) = permissions;
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO mailbox_assignments (user_id, mailbox_id, can_view, can_read, can_reply, can_send, can_send_as, can_mark_read, can_move, can_delete, can_manage, assigned_by, created_at, updated_at, revoked_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'), NULL)
               ON CONFLICT(user_id, mailbox_id) DO UPDATE SET
                   can_view = excluded.can_view,
                   can_read = excluded.can_read,
                   can_reply = excluded.can_reply,
                   can_send = excluded.can_send,
                   can_send_as = excluded.can_send_as,
                   can_mark_read = excluded.can_mark_read,
                   can_move = excluded.can_move,
                   can_delete = excluded.can_delete,
                   can_manage = excluded.can_manage,
                   assigned_by = excluded.assigned_by,
                   updated_at = datetime('now'),
                   revoked_at = NULL
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(mailbox_id)
        .bind(can_view)
        .bind(can_read)
        .bind(can_reply)
        .bind(can_send)
        .bind(can_send_as)
        .bind(can_mark_read)
        .bind(can_move)
        .bind(can_delete)
        .bind(can_manage)
        .bind(assigned_by)
        .fetch_one(self.pool)
        .await?;

        let assignment = sqlx::query_as::<_, MailboxAssignment>(
            "SELECT id, user_id, mailbox_id, can_view, can_read, can_reply, can_send, can_send_as, can_mark_read, can_move, can_delete, can_manage, assigned_by, created_at, updated_at, revoked_at FROM mailbox_assignments WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(assignment)
    }

    pub async fn list_user_mailboxes(
        &self,
        user_id: i64,
    ) -> Result<Vec<(MailcowMailbox, MailboxAssignment)>> {
        let rows = sqlx::query(
            r#"SELECT m.id as m_id, m.domain_id, m.external_id, m.address, m.local_part, m.display_name, m.active, m.quota, m.used_quota, m.credential_status, m.connection_status, m.sync_status, m.last_seen_at, m.deleted_at, m.created_at as m_created_at, m.updated_at as m_updated_at,
                      ma.id as ma_id, ma.user_id, ma.mailbox_id, ma.can_view, ma.can_read, ma.can_reply, ma.can_send, ma.can_send_as, ma.can_mark_read, ma.can_move, ma.can_delete, ma.can_manage, ma.assigned_by, ma.created_at as ma_created_at, ma.updated_at as ma_updated_at, ma.revoked_at
               FROM mailboxes m
               JOIN mailbox_assignments ma ON ma.mailbox_id = m.id
               WHERE ma.user_id = ? AND ma.revoked_at IS NULL AND m.deleted_at IS NULL
               ORDER BY m.address ASC"#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        let mut res = Vec::new();
        for row in rows {
            use sqlx::Row;
            let mb = MailcowMailbox {
                id: row.get("m_id"),
                domain_id: row.get("domain_id"),
                external_id: row.get("external_id"),
                address: row.get("address"),
                local_part: row.get("local_part"),
                display_name: row.get("display_name"),
                active: row.get("active"),
                quota: row.get("quota"),
                used_quota: row.get("used_quota"),
                credential_status: row.get("credential_status"),
                connection_status: row.get("connection_status"),
                sync_status: row.get("sync_status"),
                last_seen_at: row.get("last_seen_at"),
                deleted_at: row.get("deleted_at"),
                created_at: row.get("m_created_at"),
                updated_at: row.get("m_updated_at"),
            };
            let ma = MailboxAssignment {
                id: row.get("ma_id"),
                user_id: row.get("user_id"),
                mailbox_id: row.get("mailbox_id"),
                can_view: row.get("can_view"),
                can_read: row.get("can_read"),
                can_reply: row.get("can_reply"),
                can_send: row.get("can_send"),
                can_send_as: row.get("can_send_as"),
                can_mark_read: row.get("can_mark_read"),
                can_move: row.get("can_move"),
                can_delete: row.get("can_delete"),
                can_manage: row.get("can_manage"),
                assigned_by: row.get("assigned_by"),
                created_at: row.get("ma_created_at"),
                updated_at: row.get("ma_updated_at"),
                revoked_at: row.get("revoked_at"),
            };
            res.push((mb, ma));
        }

        Ok(res)
    }

    pub async fn list_all(&self) -> Result<Vec<MailcowMailbox>> {
        let mbs = sqlx::query_as::<_, MailcowMailbox>(
            "SELECT id, domain_id, external_id, address, local_part, display_name, active, quota, used_quota, credential_status, connection_status, sync_status, last_seen_at, deleted_at, created_at, updated_at FROM mailboxes WHERE deleted_at IS NULL ORDER BY address ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(mbs)
    }
}
