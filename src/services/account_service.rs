/// Account management service
use crate::models::account::{Account, EmailProvider};
use anyhow::Result;
use sqlx::SqlitePool;

/// Add a new email account
pub async fn add_account(
    pool: &SqlitePool,
    email: &str,
    password: &str,
    provider: EmailProvider,
    display_name: Option<String>,
    custom_config: Option<(String, u16, String, u16)>, // (imap_host, imap_port, smtp_host, smtp_port)
) -> Result<Account> {
    let id = Account::generate_id(email);

    // Check if account already exists
    let existing = sqlx::query!("SELECT id FROM accounts WHERE id = ?", id)
        .fetch_optional(pool)
        .await?;

    if existing.is_some() {
        anyhow::bail!("Account already exists: {}", email);
    }

    // Get provider config
    let config = provider.default_config();
    let (imap_host, imap_port, smtp_host, smtp_port) = if let Some((ih, ip, sh, sp)) = custom_config
    {
        (ih, ip, sh, sp)
    } else {
        (
            config.imap_host,
            config.imap_port,
            config.smtp_host,
            config.smtp_port,
        )
    };

    let credentials_encrypted = Account::encode_credentials(email, password);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let provider_str = provider.as_str();
    let display_name_str = display_name.as_deref();
    let enabled = true;
    let sync_freq: i64 = 300;
    let initial_last_uid: u32 = 0; // Fetch all emails on first sync

    sqlx::query!(
        r#"
        INSERT INTO accounts (
            id, email, provider, display_name, 
            imap_host, imap_port, smtp_host, smtp_port,
            credentials_encrypted, enabled, sync_frequency_secs,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        id,
        email,
        provider_str,
        display_name_str,
        imap_host,
        imap_port,
        smtp_host,
        smtp_port,
        credentials_encrypted,
        enabled,
        sync_freq,
        now,
        now
    )
    .execute(pool)
    .await?;

    let account = Account {
        id,
        email: email.to_string(),
        provider,
        display_name,
        imap_host,
        imap_port,
        smtp_host,
        smtp_port,
        credentials_encrypted,
        enabled: true,
        sync_frequency_secs: 300,
        last_sync_ts: Some(initial_last_uid as i64), // Set last_sync_ts to 0 for first sync
        created_at: now,
        updated_at: now,
        password: String::new(), // Will be populated on demand
    };

    Ok(account)
}

/// Get all accounts
pub async fn list_accounts(pool: &SqlitePool) -> Result<Vec<Account>> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            id, email, provider, display_name,
            imap_host, imap_port, smtp_host, smtp_port,
            credentials_encrypted, enabled, sync_frequency_secs,
            last_sync_ts, created_at, updated_at
        FROM accounts
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let accounts = rows
        .into_iter()
        .map(|row| {
            Account {
                id: row.id.unwrap_or_default(),
                email: row.email,
                provider: EmailProvider::from_str(&row.provider),
                display_name: row.display_name,
                imap_host: row.imap_host,
                imap_port: row.imap_port as u16,
                smtp_host: row.smtp_host,
                smtp_port: row.smtp_port as u16,
                credentials_encrypted: row.credentials_encrypted,
                enabled: row.enabled,
                sync_frequency_secs: row.sync_frequency_secs.unwrap_or(300),
                last_sync_ts: row.last_sync_ts,
                created_at: row.created_at,
                updated_at: row.updated_at,
                password: String::new(), // Will be populated on demand
            }
        })
        .collect();

    Ok(accounts)
}

/// Get account by ID
pub async fn get_account(pool: &SqlitePool, account_id: &str) -> Result<Option<Account>> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, email, provider, display_name,
            imap_host, imap_port, smtp_host, smtp_port,
            credentials_encrypted, enabled, sync_frequency_secs,
            last_sync_ts, created_at, updated_at
        FROM accounts
        WHERE id = ?
        "#,
        account_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(match row {
        Some(r) => {
            let mut account = Account {
                id: r.id.unwrap_or_default(),
                email: r.email,
                provider: EmailProvider::from_str(&r.provider),
                display_name: r.display_name,
                imap_host: r.imap_host,
                imap_port: r.imap_port as u16,
                smtp_host: r.smtp_host,
                smtp_port: r.smtp_port as u16,
                credentials_encrypted: r.credentials_encrypted,
                enabled: r.enabled,
                sync_frequency_secs: r.sync_frequency_secs.unwrap_or(300),
                last_sync_ts: r.last_sync_ts,
                created_at: r.created_at,
                updated_at: r.updated_at,
                password: String::new(), // populated below when available
            };

            if !account.credentials_encrypted.is_empty() {
                account = account.with_password()?;
            }

            Some(account)
        }
        None => None,
    })
}

/// Delete account
pub async fn delete_account(pool: &SqlitePool, account_id: &str) -> Result<bool> {
    let result = sqlx::query!("DELETE FROM accounts WHERE id = ?", account_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Update last sync timestamp
pub async fn update_last_sync(pool: &SqlitePool, account_id: &str) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    sqlx::query!(
        "UPDATE accounts SET last_sync_ts = ?, updated_at = ? WHERE id = ?",
        now,
        now,
        account_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
