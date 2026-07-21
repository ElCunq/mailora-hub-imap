use anyhow::Result;
use sqlx::SqlitePool;
use crate::audit::model::{AuditLog, SendAuditLog};

pub struct AuditRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AuditRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn log_action(
        &self,
        actor_user_id: Option<i64>,
        action: &str,
        resource_type: &str,
        resource_id: Option<&str>,
        domain_id: Option<i64>,
        mailbox_id: Option<i64>,
        metadata_json: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<i64> {
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO audit_logs (
                   actor_user_id, action, resource_type, resource_id,
                   domain_id, mailbox_id, metadata_json, ip_address, user_agent, created_at
               ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
               RETURNING id"#
        )
        .bind(actor_user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(domain_id)
        .bind(mailbox_id)
        .bind(metadata_json)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn list_audit_logs(
        &self,
        actor_user_id: Option<i64>,
        domain_id: Option<i64>,
        mailbox_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let mut sql = "SELECT id, actor_user_id, action, resource_type, resource_id, domain_id, mailbox_id, metadata_json, ip_address, user_agent, created_at FROM audit_logs WHERE 1=1".to_string();
        if actor_user_id.is_some() {
            sql.push_str(" AND actor_user_id = ?");
        }
        if domain_id.is_some() {
            sql.push_str(" AND domain_id = ?");
        }
        if mailbox_id.is_some() {
            sql.push_str(" AND mailbox_id = ?");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, AuditLog>(&sql);
        if let Some(uid) = actor_user_id {
            query = query.bind(uid);
        }
        if let Some(did) = domain_id {
            query = query.bind(did);
        }
        if let Some(mid) = mailbox_id {
            query = query.bind(mid);
        }
        query = query.bind(limit).bind(offset);

        let logs = query.fetch_all(self.pool).await?;
        Ok(logs)
    }

    pub async fn log_send_attempt(
        &self,
        actor_user_id: i64,
        mailbox_id: i64,
        from_address: &str,
        recipients: &str,
        message_id: Option<&str>,
        status: &str,
    ) -> Result<i64> {
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO send_audit (
                   actor_user_id, mailbox_id, from_address, recipients, message_id, status, created_at
               ) VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
               RETURNING id"#
        )
        .bind(actor_user_id)
        .bind(mailbox_id)
        .bind(from_address)
        .bind(recipients)
        .bind(message_id)
        .bind(status)
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn update_send_audit_status(
        &self,
        id: i64,
        status: &str,
        message_id: Option<&str>,
        smtp_response: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE send_audit
               SET status = ?,
                   message_id = COALESCE(?, message_id),
                   smtp_response = ?,
                   sent_at = CASE WHEN ? = 'sent' THEN datetime('now') ELSE sent_at END
               WHERE id = ?"#
        )
        .bind(status)
        .bind(message_id)
        .bind(smtp_response)
        .bind(status)
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_send_audit(
        &self,
        mailbox_id: Option<i64>,
        actor_user_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SendAuditLog>> {
        let mut sql = "SELECT id, actor_user_id, mailbox_id, from_address, recipients, message_id, smtp_response, status, created_at, sent_at FROM send_audit WHERE 1=1".to_string();
        if mailbox_id.is_some() {
            sql.push_str(" AND mailbox_id = ?");
        }
        if actor_user_id.is_some() {
            sql.push_str(" AND actor_user_id = ?");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, SendAuditLog>(&sql);
        if let Some(mid) = mailbox_id {
            query = query.bind(mid);
        }
        if let Some(uid) = actor_user_id {
            query = query.bind(uid);
        }
        query = query.bind(limit).bind(offset);

        let entries = query.fetch_all(self.pool).await?;
        Ok(entries)
    }
}
