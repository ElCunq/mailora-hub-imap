pub mod model;

use anyhow::Result;
use sqlx::SqlitePool;
use model::{DomainAdminAssignment, MailcowDomain, UpsertDomain};

pub struct DomainRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> DomainRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, req: UpsertDomain) -> Result<MailcowDomain> {
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO domains (mailcow_instance_id, external_id, name, active, quota, mailbox_limit, last_seen_at, deleted_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, datetime('now'), NULL, datetime('now'))
               ON CONFLICT(mailcow_instance_id, name) DO UPDATE SET
                   external_id = excluded.external_id,
                   active = excluded.active,
                   quota = excluded.quota,
                   mailbox_limit = excluded.mailbox_limit,
                   last_seen_at = datetime('now'),
                   deleted_at = NULL,
                   updated_at = datetime('now')
               RETURNING id"#,
        )
        .bind(req.mailcow_instance_id)
        .bind(&req.external_id)
        .bind(&req.name)
        .bind(req.active)
        .bind(req.quota)
        .bind(req.mailbox_limit)
        .fetch_one(self.pool)
        .await?;

        self.get_by_id(id).await
    }

    pub async fn get_by_id(&self, id: i64) -> Result<MailcowDomain> {
        let domain = sqlx::query_as::<_, MailcowDomain>(
            "SELECT id, mailcow_instance_id, external_id, name, active, quota, mailbox_limit, last_seen_at, deleted_at, created_at, updated_at FROM domains WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(domain)
    }

    pub async fn list_by_instance(&self, instance_id: i64) -> Result<Vec<MailcowDomain>> {
        let domains = sqlx::query_as::<_, MailcowDomain>(
            "SELECT id, mailcow_instance_id, external_id, name, active, quota, mailbox_limit, last_seen_at, deleted_at, created_at, updated_at FROM domains WHERE mailcow_instance_id = ? AND deleted_at IS NULL ORDER BY name ASC",
        )
        .bind(instance_id)
        .fetch_all(self.pool)
        .await?;

        Ok(domains)
    }

    pub async fn soft_delete_unseen(&self, instance_id: i64, seen_before: &str) -> Result<u64> {
        let res = sqlx::query(
            "UPDATE domains SET deleted_at = datetime('now'), active = 0, updated_at = datetime('now') WHERE mailcow_instance_id = ? AND last_seen_at < ? AND deleted_at IS NULL",
        )
        .bind(instance_id)
        .bind(seen_before)
        .execute(self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn assign_admin(
        &self,
        user_id: i64,
        domain_id: i64,
        assigned_by: Option<i64>,
    ) -> Result<DomainAdminAssignment> {
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO domain_admin_assignments (user_id, domain_id, assigned_by, created_at, revoked_at)
               VALUES (?, ?, ?, datetime('now'), NULL)
               ON CONFLICT(user_id, domain_id) DO UPDATE SET
                   revoked_at = NULL,
                   assigned_by = excluded.assigned_by
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(domain_id)
        .bind(assigned_by)
        .fetch_one(self.pool)
        .await?;

        let assignment = sqlx::query_as::<_, DomainAdminAssignment>(
            "SELECT id, user_id, domain_id, assigned_by, created_at, revoked_at FROM domain_admin_assignments WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(assignment)
    }

    pub async fn revoke_admin(&self, user_id: i64, domain_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE domain_admin_assignments SET revoked_at = datetime('now') WHERE user_id = ? AND domain_id = ? AND revoked_at IS NULL",
        )
        .bind(user_id)
        .bind(domain_id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_user_domains(&self, user_id: i64) -> Result<Vec<MailcowDomain>> {
        let domains = sqlx::query_as::<_, MailcowDomain>(
            r#"SELECT d.id, d.mailcow_instance_id, d.external_id, d.name, d.active, d.quota, d.mailbox_limit, d.last_seen_at, d.deleted_at, d.created_at, d.updated_at
               FROM domains d
               JOIN domain_admin_assignments daa ON daa.domain_id = d.id
               WHERE daa.user_id = ? AND daa.revoked_at IS NULL AND d.deleted_at IS NULL
               ORDER BY d.name ASC"#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        Ok(domains)
    }

    pub async fn list_all(&self) -> Result<Vec<MailcowDomain>> {
        let domains = sqlx::query_as::<_, MailcowDomain>(
            "SELECT id, mailcow_instance_id, external_id, name, active, quota, mailbox_limit, last_seen_at, deleted_at, created_at, updated_at FROM domains WHERE deleted_at IS NULL ORDER BY name ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(domains)
    }
}
