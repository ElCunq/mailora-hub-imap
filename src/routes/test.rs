/// IMAP Test Endpoints
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::services::{account_service, imap_test_service, message_body_service};

#[derive(Debug, Deserialize)]
pub struct TestQuery {
    pub limit: Option<u32>,
    pub folder: Option<String>,
}

/// GET /test/connection/:account_id - Test IMAP connection
pub async fn test_connection(
    State(pool): State<SqlitePool>,
    Path(account_id): Path<String>,
) -> Result<Json<imap_test_service::ImapConnectionTestResult>, (StatusCode, String)> {
    let account = account_service::get_account(&pool, &account_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Account {} not found", account_id)))?;
    
    tracing::info!("Testing IMAP connection for account: {}", account.email);
    
    let result = imap_test_service::test_imap_connection(&account)
        .await
        .map_err(|e| {
            tracing::error!("IMAP connection test failed: {}", e);
            
            // Provider-specific error messages
            let error_msg = e.to_string();
            let helpful_msg = if error_msg.contains("login failed") || error_msg.contains("LOGIN failed") {
                match account.provider.as_str() {
                    "gmail" => {
                        "Gmail bağlantı hatası:\n\
                        • 2-Step Verification aktif olmalı\n\
                        • App Password kullanın: https://myaccount.google.com/apppasswords\n\
                        • IMAP aktif olmalı (Gmail Settings > Forwarding and POP/IMAP)"
                    },
                    "outlook" => {
                        "Outlook/Hotmail bağlantı hatası:\n\
                        • 2-Step Verification aktif olmalı\n\
                        • App Password kullanın: https://account.microsoft.com/security\n\
                        • IMAP aktif olmalı (Outlook.com > Settings > Sync email)\n\
                        • Host: outlook.office365.com, Port: 993"
                    },
                    "yahoo" => {
                        "Yahoo bağlantı hatası:\n\
                        • App Password kullanın: https://login.yahoo.com/account/security\n\
                        • IMAP aktif olmalı"
                    },
                    _ => &error_msg
                }
            } else {
                &error_msg
            };
            
            (StatusCode::BAD_REQUEST, helpful_msg.to_string())
        })?;
    
    tracing::info!("IMAP connection successful. Folders: {}, Messages: {}", 
        result.folders.len(), 
        result.inbox_stats.exists
    );
    
    Ok(Json(result))
}

/// GET /test/messages/:account_id - Fetch recent messages preview
pub async fn fetch_messages(
    State(pool): State<SqlitePool>,
    Path(account_id): Path<String>,
    Query(query): Query<TestQuery>,
) -> Result<Json<FetchMessagesResponse>, (StatusCode, String)> {
    let account = account_service::get_account(&pool, &account_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Account {} not found", account_id)))?;
    
    let limit = query.limit.unwrap_or(10).min(50); // Max 50 messages
    
    tracing::info!("Fetching {} recent messages for account: {}", limit, account.email);
    
    let messages = imap_test_service::fetch_recent_messages(&account, limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch messages: {}", e);
            
            // Provider-specific error messages
            let error_msg = e.to_string();
            let helpful_msg = if error_msg.contains("LOGIN failed") {
                match account.provider.as_str() {
                    "gmail" => {
                        "LOGIN başarısız. Gmail için:\n\
                        1. 2-Step Verification aktif olmalı\n\
                        2. App Password oluşturun: https://myaccount.google.com/apppasswords\n\
                        3. Normal şifre yerine App Password kullanın"
                    },
                    "outlook" => {
                        "LOGIN başarısız. Outlook/Hotmail için:\n\
                        1. 2-Step Verification aktif olmalı\n\
                        2. App Password oluşturun: https://account.microsoft.com/security\n\
                        3. Normal şifre yerine App Password kullanın\n\
                        4. IMAP erişimi aktif olmalı (Settings > Sync email)"
                    },
                    "yahoo" => {
                        "LOGIN başarısız. Yahoo için:\n\
                        1. App Password oluşturun: https://login.yahoo.com/account/security\n\
                        2. Normal şifre yerine App Password kullanın"
                    },
                    _ => "LOGIN başarısız. Lütfen:\n\
                        1. Email ve şifrenizi kontrol edin\n\
                        2. IMAP erişimi aktif olmalı\n\
                        3. Büyük provider'lar için App Password gereklidir"
                }
            } else {
                &error_msg
            };
            
            (StatusCode::BAD_REQUEST, helpful_msg.to_string())
        })?;
    
    tracing::info!("Fetched {} messages", messages.len());
    
    Ok(Json(FetchMessagesResponse {
        account_id: account.id,
        email: account.email,
        message_count: messages.len(),
        messages,
    }))
}

/// GET /test/accounts - List all test accounts
pub async fn list_test_accounts(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<TestAccountInfo>>, (StatusCode, String)> {
    let accounts = account_service::list_accounts(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    let test_info: Vec<TestAccountInfo> = accounts
        .into_iter()
        .map(|acc| TestAccountInfo {
            id: acc.id,
            email: acc.email,
            provider: acc.provider.as_str().to_string(),
            enabled: acc.enabled,
            last_sync_ts: acc.last_sync_ts,
        })
        .collect();
    
    Ok(Json(test_info))
}

/// GET /test/body/:account_id/:uid - Fetch full message body
pub async fn fetch_message_body(
    State(pool): State<SqlitePool>,
    Path((account_id, uid)): Path<(String, u32)>,
    Query(query): Query<TestQuery>,
) -> Result<Json<message_body_service::MessageBody>, (StatusCode, String)> {
    let account = account_service::get_account(&pool, &account_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Account {} not found", account_id)))?;
    
    tracing::info!("Fetching body for message {} from account: {}", uid, account.email);
    
    let folder = query.folder.as_deref();
    let body = message_body_service::fetch_message_body(&account, uid, folder)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch message body: {}", e);
            (StatusCode::BAD_REQUEST, format!("Fetch failed: {}", e))
        })?;
    
    tracing::info!("Fetched message body: {} bytes", body.raw_size);
    
    Ok(Json(body))
}

#[derive(Debug, Serialize)]
pub struct FetchMessagesResponse {
    pub account_id: String,
    pub email: String,
    pub message_count: usize,
    pub messages: Vec<imap_test_service::MessagePreview>,
}

#[derive(Debug, Serialize)]
pub struct TestAccountInfo {
    pub id: String,
    pub email: String,
    pub provider: String,
    pub enabled: bool,
    pub last_sync_ts: Option<i64>,
}
