use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct MailcowDomain {
    pub id: i64,
    pub mailcow_instance_id: i64,
    pub external_id: Option<String>,
    pub name: String,
    pub active: bool,
    pub quota: Option<i64>,
    pub mailbox_limit: Option<i64>,
    pub last_seen_at: String,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertDomain {
    pub mailcow_instance_id: i64,
    pub external_id: Option<String>,
    pub name: String,
    pub active: bool,
    pub quota: Option<i64>,
    pub mailbox_limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct DomainAdminAssignment {
    pub id: i64,
    pub user_id: i64,
    pub domain_id: i64,
    pub assigned_by: Option<i64>,
    pub created_at: String,
    pub revoked_at: Option<String>,
}
