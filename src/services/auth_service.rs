use crate::models::user::{CreateUserReq, User};
use anyhow::Result;
use sqlx::SqlitePool;
use bcrypt::{hash, verify, DEFAULT_COST};

pub async fn auto_assign_user_domain(pool: &SqlitePool, user_id: i64, username: &str) {
    if let Some(domain_part) = username.split('@').nth(1) {
        let domain_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM domains WHERE name = ? AND deleted_at IS NULL"
        )
        .bind(domain_part)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        if let Some(did) = domain_id {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO user_domain_assignments (user_id, domain_id, created_at)
                 VALUES (?, ?, datetime('now'))"
            )
            .bind(user_id)
            .bind(did)
            .execute(pool)
            .await;
        }
    }
}

pub async fn register_user(pool: &SqlitePool, req: CreateUserReq) -> Result<User> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    
    let role = if count == 0 { "SuperAdmin" } else { "User" };
    let hash_str = hash(req.password, DEFAULT_COST)?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, password_hash, role, created_at, updated_at) 
         VALUES (?, ?, ?, datetime('now'), datetime('now')) RETURNING id"
    )
    .bind(&req.username)
    .bind(&hash_str)
    .bind(role)
    .fetch_one(pool)
    .await?;

    // Auto-assign to domain group if username is an email address
    auto_assign_user_domain(pool, id, &req.username).await;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

pub async fn verify_user(pool: &SqlitePool, username: &str, password: &str) -> Result<Option<User>> {
    // 1. Try local users database authentication first
    let user_opt = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    if let Some(user) = user_opt {
        if verify(password, &user.password_hash).is_ok() {
            return Ok(Some(user));
        }
    }

    // 2. Fallback: Authenticate directly against Mailcow IMAP if it is a mailbox address
    if username.contains('@') {
        #[derive(sqlx::FromRow)]
        struct MailboxMeta {
            mailbox_id: i64,
            imap_host: String,
            imap_port: i64,
        }

        let meta_opt = sqlx::query_as::<_, MailboxMeta>(
            "SELECT m.id as mailbox_id, mi.imap_host, mi.imap_port 
             FROM mailboxes m
             JOIN domains d ON m.domain_id = d.id
             JOIN mailcow_instances mi ON d.mailcow_instance_id = mi.id
             WHERE m.address = ? AND m.active = 1"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        if let Some(meta) = meta_opt {
            let port = meta.imap_port as u16;
            if let Ok(mut imap) = crate::imap::conn::connect(&meta.imap_host, port, username, password).await {
                let _ = imap.session.logout().await;

                // Success! Auto-onboard or update password hash locally.
                let hash_str = hash(password, DEFAULT_COST)?;
                
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO users (username, password_hash, role, created_at, updated_at) 
                     VALUES (?, ?, 'User', datetime('now'), datetime('now'))
                     ON CONFLICT(username) DO UPDATE SET password_hash = excluded.password_hash, updated_at = datetime('now')
                     RETURNING id"
                )
                .bind(username)
                .bind(&hash_str)
                .fetch_one(pool)
                .await?;

                // Auto-assign to domain group if username is an email address
                auto_assign_user_domain(pool, id, username).await;

                let encrypted = crate::services::crypto::encrypt_secret(password);
                let _ = sqlx::query(
                    "INSERT INTO mailbox_credentials (mailbox_id, username, password_encrypted, created_at, updated_at) 
                     VALUES (?, ?, ?, datetime('now'), datetime('now'))
                     ON CONFLICT(mailbox_id) DO UPDATE SET password_encrypted = excluded.password_encrypted, updated_at = datetime('now')"
                )
                .bind(meta.mailbox_id)
                .bind(username)
                .bind(&encrypted)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    "INSERT INTO mailbox_assignments (user_id, mailbox_id, can_view, can_read, can_reply, can_send, can_send_as, can_mark_read, can_move, can_delete, can_manage, created_at, updated_at)
                     VALUES (?, ?, 1, 1, 1, 1, 1, 1, 1, 1, 1, datetime('now'), datetime('now'))
                     ON CONFLICT(user_id, mailbox_id) DO NOTHING"
                )
                .bind(id)
                .bind(meta.mailbox_id)
                .execute(pool)
                .await;

                let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
                    .bind(id)
                    .fetch_one(pool)
                    .await?;

                return Ok(Some(user));
            }
        }
    }

    Ok(None)
}
