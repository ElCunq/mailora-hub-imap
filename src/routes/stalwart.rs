/// Stalwart integration endpoints - DEPRECATED
/// This module is no longer used. Direct IMAP/SMTP is now used instead.
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
// use crate::stalwart_client;

#[derive(Debug, Deserialize)]
pub struct StalwartSyncRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct StalwartSyncResponse {
    pub success: bool,
    pub account_id: String,
    pub synced_count: usize,
    pub message: String,
}

/// POST /stalwart/sync - Sync messages from Stalwart IMAP server
/// DEPRECATED: This endpoint is no longer supported. Use /sync/:account_id instead.
pub async fn sync_stalwart(
    State(_pool): State<SqlitePool>,
    Json(_req): Json<StalwartSyncRequest>,
) -> Result<Json<StalwartSyncResponse>, StatusCode> {
    Ok(Json(StalwartSyncResponse {
        success: false,
        account_id: String::new(),
        synced_count: 0,
        message: "DEPRECATED: Stalwart integration removed. Use direct IMAP sync via /sync/:account_id".to_string(),
    }))
}
            Ok(Json(StalwartSyncResponse {
                success: true,
                account_id,
                synced_count: count,
                message: format!("Successfully synced {} messages", count),
            }))
        }
        Err(e) => {
            tracing::error!("Stalwart sync error: {}", e);
            Ok(Json(StalwartSyncResponse {
                success: false,
                account_id: String::new(),
                synced_count: 0,
                message: format!("Sync error: {}", e),
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StalwartTestRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct StalwartTestResponse {
    pub connected: bool,
    pub message: String,
}

/// POST /stalwart/test - Test Stalwart IMAP connection
/// DEPRECATED: This endpoint is no longer supported. Use /test/connection/:account_id instead.
pub async fn test_stalwart(
    Json(_req): Json<StalwartTestRequest>,
) -> Result<Json<StalwartTestResponse>, StatusCode> {
    Ok(Json(StalwartTestResponse {
        connected: false,
        message: "DEPRECATED: Stalwart integration removed. Use /test/connection/:account_id".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct StalwartConfigResponse {
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub admin_url: String,
}

/// GET /stalwart/config - Get Stalwart configuration
pub async fn get_stalwart_config() -> Result<Json<StalwartConfigResponse>, StatusCode> {
    let config = stalwart_client::StalwartConfig::default();
    Ok(Json(StalwartConfigResponse {
        imap_host: config.imap_host,
        imap_port: config.imap_port,
        smtp_host: config.smtp_host,
        smtp_port: config.smtp_port,
        admin_url: config.admin_url,
    }))
}
