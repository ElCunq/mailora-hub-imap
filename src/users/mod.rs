pub mod model;

use anyhow::Result;
use sqlx::SqlitePool;
use model::{CreateUserRequest, MailoraUser};

pub struct UserRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateUserRequest) -> Result<MailoraUser> {
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO users (email, username, password_hash, role, created_at, updated_at)
               VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))
               RETURNING id"#,
        )
        .bind(&req.email)
        .bind(&req.email)
        .bind(&req.password_hash)
        .bind(&req.role)
        .fetch_one(self.pool)
        .await?;

        self.get_by_id(id).await
    }

    pub async fn get_by_id(&self, id: i64) -> Result<MailoraUser> {
        let user = sqlx::query_as::<_, MailoraUser>(
            "SELECT id, email, username, password_hash, role FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_by_email(&self, email: &str) -> Result<MailoraUser> {
        let user = sqlx::query_as::<_, MailoraUser>(
            "SELECT id, email, username, password_hash, role FROM users WHERE email = ? OR username = ?",
        )
        .bind(email)
        .bind(email)
        .fetch_one(self.pool)
        .await?;

        Ok(user)
    }

    pub async fn list_all(&self) -> Result<Vec<MailoraUser>> {
        let users = sqlx::query_as::<_, MailoraUser>(
            "SELECT id, email, username, password_hash, role FROM users ORDER BY id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(users)
    }

    pub async fn update_role(&self, id: i64, role: &str) -> Result<MailoraUser> {
        sqlx::query("UPDATE users SET role = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(role)
            .bind(id)
            .execute(self.pool)
            .await?;

        self.get_by_id(id).await
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
