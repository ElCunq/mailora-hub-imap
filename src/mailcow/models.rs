use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct MailcowInstance {
    pub id: i64,
    pub name: String,
    pub base_url: String,
    pub api_key_encrypted: String,
    pub imap_host: String,
    pub imap_port: i64,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub enabled: bool,
    pub last_discovery_at: Option<String>,
    pub last_discovery_status: Option<String>,
    pub last_discovery_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMailcowInstance {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub api_key_encrypted: Option<String>,
    pub imap_host: String,
    pub imap_port: i64,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateMailcowInstance {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key_encrypted: Option<String>,
    pub imap_host: Option<String>,
    pub imap_port: Option<i64>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i64>,
    pub enabled: Option<bool>,
}
