use std::time::Duration;
use sqlx::SqlitePool;
use tracing::{info, warn};

/// Başlangıçta MAILCOW_URL + MAILCOW_API_KEY env var'ları varsa
/// ve veritabanında hiç instance yoksa otomatik olarak ilk instance'ı kaydet.
pub async fn seed_mailcow_instance_from_env(pool: &SqlitePool) {
    let base_url = std::env::var("MAILCOW_URL").unwrap_or_default();
    let api_key = std::env::var("MAILCOW_API_KEY").unwrap_or_default();

    if base_url.is_empty() || api_key.is_empty() {
        // Env var tanımlı değil, atla
        return;
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mailcow_instances")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if count > 0 {
        info!("Mailcow instance already exists in DB, skipping env-based seed.");
        return;
    }

    // Env'deki URL'den IMAP/SMTP host çıkar (aynı hostname varsayılan)
    let host = base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string();

    let imap_host = std::env::var("MAILCOW_IMAP_HOST").unwrap_or_else(|_| host.clone());
    let smtp_host = std::env::var("MAILCOW_SMTP_HOST").unwrap_or_else(|_| host.clone());
    let imap_port: i64 = std::env::var("MAILCOW_IMAP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(993);
    let smtp_port: i64 = std::env::var("MAILCOW_SMTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(587);

    let encrypted_key = crate::services::crypto::encrypt_secret(&api_key);

    let res = sqlx::query(
        "INSERT INTO mailcow_instances (name, base_url, api_key_encrypted, imap_host, imap_port, smtp_host, smtp_port, enabled, created_at, updated_at)
         VALUES ('Mailcow (Auto)', ?, ?, ?, ?, ?, ?, 1, datetime('now'), datetime('now'))"
    )
    .bind(&base_url)
    .bind(&encrypted_key)
    .bind(&imap_host)
    .bind(imap_port)
    .bind(&smtp_host)
    .bind(smtp_port)
    .execute(pool)
    .await;

    match res {
        Ok(_) => info!(base_url = %base_url, imap = %imap_host, "✅ Mailcow instance auto-seeded from environment variables"),
        Err(e) => warn!(error = %e, "Failed to auto-seed Mailcow instance from env"),
    }
}

/// Arka planda her 30 saniyede bir Mailcow keşfi çalıştırır.
pub fn start(pool: SqlitePool) {
    let discovery_pool = pool.clone();
    tokio::spawn(async move {
        let service = crate::mailcow::discovery::DiscoveryService::new(&discovery_pool);
        loop {
            match service.run_discovery_all().await {
                Ok(summaries) => {
                    if summaries.is_empty() {
                        info!("Mailcow discovery: no instances configured yet. Add one from the admin panel or set MAILCOW_URL + MAILCOW_API_KEY env vars.");
                    } else {
                        info!(instances = summaries.len(), "Periodic Mailcow discovery completed successfully");
                    }
                }
                Err(e) => {
                    warn!(error = %e.to_string(), "Periodic Mailcow discovery encountered error");
                }
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}
