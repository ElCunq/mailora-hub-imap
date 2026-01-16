#[derive(Debug, Serialize)]
pub struct AddAccountResponse {
    pub success: bool,
    pub account_id: String,
    pub message: String,
}
/// POST /accounts - Add a new email account
pub async fn add_account(
    State(pool): State<SqlitePool>,
    Json(req): Json<AddAccountRequest>,
) -> Json<AddAccountResponse> {
    let provider = EmailProvider::from_str(&req.provider);
    // Password flow - validate password
    if req.password.is_none() {
        return Json(AddAccountResponse {
            success: false,
            account_id: String::new(),
            message: "Password is required for authentication".to_string(),
        });
    }
    // Validate custom provider
    let custom_config = if provider == EmailProvider::Custom {
        if req.imap_host.is_none() || req.smtp_host.is_none() {
            return Json(AddAccountResponse {
                success: false,
                account_id: String::new(),
                message: "Custom provider requires imap_host and smtp_host".to_string(),
            });
        }
        Some((
            req.imap_host.clone().unwrap(),
            req.imap_port.unwrap_or(993),
            req.smtp_host.clone().unwrap(),
            req.smtp_port.unwrap_or(587),
        ))
    } else {
        None
    };
    match account_service::add_account(
        &pool,
        &req.email,
        &req.password.clone().unwrap(),
        provider,
        req.display_name.clone(),
        custom_config,
    )
    .await
    {
        Ok(account) => {
            tracing::info!("Account added: {}", account.email);
            Json(AddAccountResponse {
                success: true,
                account_id: account.id,
                message: format!("Account {} added successfully", account.email),
            })
        }
        Err(e) => {
            tracing::error!("Failed to add account: {}", e);
            Json(AddAccountResponse {
                success: false,
                account_id: String::new(),
                message: format!("Failed to add account: {}", e),
            })
        }
    }
}
#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub email: String,
    pub provider: String,
    pub display_name: Option<String>,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub enabled: bool,
    pub last_sync_ts: Option<i64>,
}

impl From<Account> for AccountResponse {
    fn from(acc: Account) -> Self {
        Self {
            id: acc.id,
            email: acc.email,
            provider: acc.provider.as_str().to_string(),
            display_name: acc.display_name,
            imap_host: acc.imap_host,
            imap_port: acc.imap_port,
            smtp_host: acc.smtp_host,
            smtp_port: acc.smtp_port,
            enabled: acc.enabled,
            last_sync_ts: acc.last_sync_ts,
        }
    }
}
use crate::models::account::{Account, EmailProvider};
use crate::services::account_service;
/// Account management endpoints
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct AddAccountRequest {
    pub email: String,
    pub password: Option<String>,
    pub provider: String,
    pub display_name: Option<String>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub provider: Option<String>,
    pub display_name: Option<Option<String>>, // Some(Some(v)) set, Some(None) clear
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub enabled: Option<bool>,
    pub append_policy: Option<Option<String>>, // Some(Some(v)) or Some(None)
    pub sent_folder_hint: Option<Option<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateAccountResponse {
    pub success: bool,
    pub account: Option<AccountResponse>,
    pub error: Option<String>,
}

/// Helper: Add OAuth2 account
async fn add_oauth_account(
    pool: &SqlitePool,
    email: &str,
    provider: EmailProvider,
    display_name: Option<String>,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
) -> Result<Account, String> {
    let account_id = uuid::Uuid::new_v4().to_string();
    let display = display_name.unwrap_or_else(|| format!("{} Account", email));

    let (imap_host, imap_port, smtp_host, smtp_port) = match provider {
        EmailProvider::Gmail => (
            "imap.gmail.com".to_string(),
            993,
            "smtp.gmail.com".to_string(),
            587,
        ),
        EmailProvider::Outlook => (
            "outlook.office365.com".to_string(),
            993,
            "smtp.office365.com".to_string(),
            587,
        ),
        EmailProvider::Yahoo => (
            "imap.mail.yahoo.com".to_string(),
            993,
            "smtp.mail.yahoo.com".to_string(),
            587,
        ),
        EmailProvider::Icloud => (
            "imap.mail.me.com".to_string(),
            993,
            "smtp.mail.me.com".to_string(),
            587,
        ),
        _ => return Err("OAuth2 not supported for this provider".to_string()),
    };

    // Insert account with OAuth2 credentials
    sqlx::query(
        r#"
        INSERT INTO accounts (
            id, email, provider, display_name,
            imap_host, imap_port, smtp_host, smtp_port,
            auth_method, oauth_access_token, oauth_refresh_token, oauth_expires_at,
            credentials_encrypted, enabled, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'oauth2', ?, ?, ?, '', 1, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&account_id)
    .bind(email)
    .bind(provider.as_str())
    .bind(&display)
    .bind(&imap_host)
    .bind(imap_port as i64)
    .bind(&smtp_host)
    .bind(smtp_port as i64)
    .bind(&access_token)
    .bind(&refresh_token)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(Account {
        id: account_id,
        email: email.to_string(),
        provider,
        display_name: Some(display),
        imap_host,
        imap_port,
        smtp_host,
        smtp_port,
        credentials_encrypted: String::new(),
        enabled: true,
        sync_frequency_secs: 300,
        last_sync_ts: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
        append_policy: None,
        sent_folder_hint: None,
        password: String::new(),
    })
}

/// GET /accounts - List all accounts
pub async fn list_accounts(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<AccountResponse>>, StatusCode> {
    match account_service::list_accounts(&pool).await {
        Ok(accounts) => {
            let response: Vec<AccountResponse> = accounts.into_iter().map(Into::into).collect();
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to list accounts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /accounts/:id - Get account by ID
pub async fn get_account(
    State(pool): State<SqlitePool>,
    Path(account_id): Path<String>,
) -> Result<Json<AccountResponse>, StatusCode> {
    match account_service::get_account(&pool, &account_id).await {
        Ok(Some(account)) => Ok(Json(account.into())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// DELETE /accounts/:id - Delete account
pub async fn delete_account(
    State(pool): State<SqlitePool>,
    Path(account_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match account_service::delete_account(&pool, &account_id).await {
        Ok(true) => {
            tracing::info!("Account deleted: {}", account_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /providers - List available email providers with configs
#[derive(Debug, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
}

pub async fn list_providers() -> Json<Vec<ProviderInfo>> {
    let providers = vec![
        EmailProvider::Gmail,
        EmailProvider::Outlook,
        EmailProvider::Yahoo,
        EmailProvider::Icloud,
    ];

    let info: Vec<ProviderInfo> = providers
        .into_iter()
        .map(|p| {
            let config = p.default_config();
            ProviderInfo {
                id: p.as_str().to_string(),
                name: match p {
                    EmailProvider::Gmail => "Gmail".to_string(),
                    EmailProvider::Outlook => "Outlook / Office 365".to_string(),
                    EmailProvider::Yahoo => "Yahoo Mail".to_string(),
                    EmailProvider::Icloud => "iCloud Mail".to_string(),
                    EmailProvider::Custom => "Custom".to_string(),
                },
                imap_host: config.imap_host,
                imap_port: config.imap_port,
                smtp_host: config.smtp_host,
                smtp_port: config.smtp_port,
            }
        })
        .collect();

    Json(info)
}
#[derive(Debug, Deserialize)]
pub struct PatchAccountRequest {
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub password: Option<String>,
    pub append_policy: Option<String>, // auto|never|force
    pub sent_folder_hint: Option<String>,
}

/// PATCH /accounts/:id - Update mutable account settings
pub async fn patch_account(
    State(pool): State<SqlitePool>,
    Path(account_id): Path<String>,
    Json(req): Json<PatchAccountRequest>,
) -> Result<Json<AccountResponse>, StatusCode> {
    // Load existing
    let existing = match account_service::get_account(&pool, &account_id).await {
        Ok(Some(acc)) => acc,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    // Validate append_policy if provided
    if let Some(ref ap) = req.append_policy {
        let v = ap.to_lowercase();
        if !["auto","never","force"].contains(&v.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    // Determine new values (fallback to existing)
    let new_display_name = req.display_name.or(existing.display_name.clone());
    let new_enabled = req.enabled.unwrap_or(existing.enabled);
    let new_imap_host = req.imap_host.unwrap_or(existing.imap_host.clone());
    let new_imap_port = req.imap_port.unwrap_or(existing.imap_port);
    let new_smtp_host = req.smtp_host.unwrap_or(existing.smtp_host.clone());
    let new_smtp_port = req.smtp_port.unwrap_or(existing.smtp_port);
    let new_append_policy = req.append_policy.as_ref().map(|s| s.to_lowercase());
    let new_sent_folder_hint = req.sent_folder_hint.or(existing.sent_folder_hint.clone());

    // Credentials: if password changed re-encode; we keep email immutable here
    let new_creds_enc = if let Some(pass) = req.password.as_ref() {
        crate::models::account::Account::encode_credentials(&existing.email, pass)
    } else {
        existing.credentials_encrypted.clone()
    };

    // Persist update (provider immutable for now)
    let res = sqlx::query(
        "UPDATE accounts SET display_name = ?, imap_host = ?, imap_port = ?, smtp_host = ?, smtp_port = ?, enabled = ?, append_policy = ?, sent_folder_hint = ?, credentials_encrypted = ?, updated_at = strftime('%s','now') WHERE id = ?"
    )
    .bind(&new_display_name)
    .bind(&new_imap_host)
    .bind(new_imap_port as i64)
    .bind(&new_smtp_host)
    .bind(new_smtp_port as i64)
    .bind(new_enabled as i64)
    .bind(&new_append_policy)
    .bind(&new_sent_folder_hint)
    .bind(&new_creds_enc)
    .bind(&account_id)
    .execute(&pool)
    .await;
    if res.is_err() { return Err(StatusCode::INTERNAL_SERVER_ERROR); }

    // Reload updated account
    let updated = match account_service::get_account(&pool, &account_id).await {
        Ok(Some(acc)) => acc,
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // If in-memory creds exist update them (host/port/password changes)
    {
        use crate::services::diff_service::ACCOUNTS;
        if let Ok((email, password)) = updated.get_credentials() {
            let mut store = ACCOUNTS.write().await;
            if let Some(entry) = store.get_mut(&account_id) {
                entry.email = email;
                entry.password = password;
                entry.host = new_imap_host;
                entry.port = new_imap_port;
            } else {
                // Optionally insert if not present and enabled
                if new_enabled {
                    store.insert(account_id.clone(), crate::services::diff_service::AccountCreds { email, password, host: new_imap_host, port: new_imap_port });
                }
            }
        }
    }

    Ok(Json(updated.into()))
}
