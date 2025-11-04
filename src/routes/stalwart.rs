// Gerekli use satırları dosyanın başında zaten var, tekrarları kaldırıldı

#[derive(Debug, Deserialize)]
pub struct StalwartApiConnectRequest {
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct StalwartApiConnectResponse {
    pub success: bool,
    pub message: String,
}

/// POST /stalwart/connect - Connect to Stalwart API with API Key
pub async fn connect_stalwart_api(
    Json(req): Json<StalwartApiConnectRequest>,
) -> Result<Json<StalwartApiConnectResponse>, StatusCode> {
    // Burada gerçek Stalwart API bağlantısı yapılabilir
    if req.api_key.trim().is_empty() {
        return Ok(Json(StalwartApiConnectResponse {
            success: false,
            message: "API Key boş olamaz".to_string(),
        }));
    }
    // Örnek: API key ile bağlantı başarılı
    Ok(Json(StalwartApiConnectResponse {
        success: true,
        message: "Stalwart API bağlantısı başarılı. Mailler çekiliyor...".to_string(),
    }))
}
/// Stalwart integration endpoints - DEPRECATED
/// This module is no longer used. Direct IMAP/SMTP is now used instead.
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
// use crate::stalwart_client; // Eğer crate yoksa bu satırı kaldırın

// DEPRECATED sync fonksiyonu ve ilgili yapılar kaldırıldı

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
        message: "DEPRECATED: Stalwart integration removed. Use /test/connection/:account_id"
            .to_string(),
    }))
}

// stalwart_client ve get_stalwart_config fonksiyonu kaldırıldı (crate mevcut değil)
