pub mod client;
pub mod discovery;
pub mod models;

use anyhow::Result;
use sqlx::SqlitePool;
use models::{CreateMailcowInstance, MailcowInstance, UpdateMailcowInstance};

pub struct MailcowRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> MailcowRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateMailcowInstance) -> Result<MailcowInstance> {
        let enabled = req.enabled.unwrap_or(true);
        let key_encrypted = if let Some(ref raw) = req.api_key {
            crate::services::crypto::encrypt_secret(raw)
        } else {
            req.api_key_encrypted.clone().unwrap_or_default()
        };
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO mailcow_instances (name, base_url, api_key_encrypted, imap_host, imap_port, smtp_host, smtp_port, enabled)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)
               RETURNING id"#,
        )
        .bind(&req.name)
        .bind(&req.base_url)
        .bind(&key_encrypted)
        .bind(&req.imap_host)
        .bind(req.imap_port)
        .bind(&req.smtp_host)
        .bind(req.smtp_port)
        .bind(enabled)
        .fetch_one(self.pool)
        .await?;

        self.get_by_id(id).await
    }

    pub async fn get_by_id(&self, id: i64) -> Result<MailcowInstance> {
        let instance = sqlx::query_as::<_, MailcowInstance>(
            "SELECT id, name, base_url, api_key_encrypted, imap_host, imap_port, smtp_host, smtp_port, enabled, last_discovery_at, last_discovery_status, last_discovery_error, created_at, updated_at FROM mailcow_instances WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(instance)
    }

    pub async fn list_all(&self) -> Result<Vec<MailcowInstance>> {
        let instances = sqlx::query_as::<_, MailcowInstance>(
            "SELECT id, name, base_url, api_key_encrypted, imap_host, imap_port, smtp_host, smtp_port, enabled, last_discovery_at, last_discovery_status, last_discovery_error, created_at, updated_at FROM mailcow_instances ORDER BY id DESC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(instances)
    }

    pub async fn update(&self, id: i64, req: UpdateMailcowInstance) -> Result<MailcowInstance> {
        let current = self.get_by_id(id).await?;
        let name = req.name.unwrap_or(current.name);
        let base_url = req.base_url.unwrap_or(current.base_url);
        let api_key_encrypted = req.api_key_encrypted.unwrap_or(current.api_key_encrypted);
        let imap_host = req.imap_host.unwrap_or(current.imap_host);
        let imap_port = req.imap_port.unwrap_or(current.imap_port);
        let smtp_host = req.smtp_host.unwrap_or(current.smtp_host);
        let smtp_port = req.smtp_port.unwrap_or(current.smtp_port);
        let enabled = req.enabled.unwrap_or(current.enabled);

        sqlx::query(
            r#"UPDATE mailcow_instances
               SET name = ?, base_url = ?, api_key_encrypted = ?, imap_host = ?, imap_port = ?, smtp_host = ?, smtp_port = ?, enabled = ?, updated_at = datetime('now')
               WHERE id = ?"#,
        )
        .bind(&name)
        .bind(&base_url)
        .bind(&api_key_encrypted)
        .bind(&imap_host)
        .bind(imap_port)
        .bind(&smtp_host)
        .bind(smtp_port)
        .bind(enabled)
        .bind(id)
        .execute(self.pool)
        .await?;

        self.get_by_id(id).await
    }

    pub async fn update_discovery_status(
        &self,
        id: i64,
        status: &str,
        error: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE mailcow_instances
               SET last_discovery_at = datetime('now'), last_discovery_status = ?, last_discovery_error = ?, updated_at = datetime('now')
               WHERE id = ?"#,
        )
        .bind(status)
        .bind(error)
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM mailcow_instances WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
