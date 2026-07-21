use anyhow::Result;
use sqlx::SqlitePool;
use crate::audit::model::{AuditLog, SendAuditLog};
use crate::audit::repository::AuditRepository;

pub struct AuditService<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AuditService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn record_action(
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
        let repo = AuditRepository::new(self.pool);
        repo.log_action(
            actor_user_id,
            action,
            resource_type,
            resource_id,
            domain_id,
            mailbox_id,
            metadata_json,
            ip_address,
            user_agent,
        )
        .await
    }

    pub async fn list_logs(
        &self,
        actor_user_id: Option<i64>,
        domain_id: Option<i64>,
        mailbox_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let repo = AuditRepository::new(self.pool);
        repo.list_audit_logs(actor_user_id, domain_id, mailbox_id, limit, offset).await
    }

    pub async fn record_send_attempt(
        &self,
        actor_user_id: i64,
        mailbox_id: i64,
        from_address: &str,
        recipients: &str,
        message_id: Option<&str>,
        status: &str,
    ) -> Result<i64> {
        let repo = AuditRepository::new(self.pool);
        repo.log_send_attempt(actor_user_id, mailbox_id, from_address, recipients, message_id, status).await
    }

    pub async fn update_send_status(
        &self,
        id: i64,
        status: &str,
        message_id: Option<&str>,
        smtp_response: Option<&str>,
    ) -> Result<()> {
        let repo = AuditRepository::new(self.pool);
        repo.update_send_audit_status(id, status, message_id, smtp_response).await
    }

    pub async fn list_send_audit(
        &self,
        mailbox_id: Option<i64>,
        actor_user_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SendAuditLog>> {
        let repo = AuditRepository::new(self.pool);
        repo.list_send_audit(mailbox_id, actor_user_id, limit, offset).await
    }
}
