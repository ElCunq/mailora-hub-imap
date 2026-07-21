use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub actor_user_id: Option<i64>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub domain_id: Option<i64>,
    pub mailbox_id: Option<i64>,
    pub metadata_json: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SendAuditLog {
    pub id: i64,
    pub actor_user_id: i64,
    pub mailbox_id: i64,
    pub from_address: String,
    pub recipients: String,
    pub message_id: Option<String>,
    pub smtp_response: Option<String>,
    pub status: String,
    pub created_at: String,
    pub sent_at: Option<String>,
}
