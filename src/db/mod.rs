// filepath: /mailora-hub-imap/mailora-hub-imap/src/db/mod.rs
use anyhow::Result;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir("migrations")?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("sql") {
            let sql = fs::read_to_string(&p)?;
            sqlx::query(&sql).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn seed_account(pool: &SqlitePool) -> Result<()> {
    let email = std::env::var("IMAP_EMAIL")?;
    let host = std::env::var("IMAP_HOST")?;
    let port: i64 = std::env::var("IMAP_PORT")?.parse()?;
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or(host.clone());
    let smtp_port: i64 = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(587);
    let now = now_epoch();
    sqlx::query(r#"INSERT OR IGNORE INTO accounts(
        id, org_id, email, imap_host, imap_port, smtp_host, smtp_port, auth_type, use_ssl, created_at, updated_at
    ) VALUES (1,1,?,?,?,?,?,'LOGIN',1,?,?)"#)
        .bind(&email)
        .bind(&host)
        .bind(port)
        .bind(&smtp_host)
        .bind(smtp_port)
        .bind(now)
        .bind(now)
        .execute(pool).await?;
    Ok(())
}

pub fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
