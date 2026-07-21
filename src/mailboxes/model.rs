use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct MailcowMailbox {
    pub id: i64,
    pub domain_id: i64,
    pub external_id: Option<String>,
    pub address: String,
    pub local_part: String,
    pub display_name: Option<String>,
    pub active: bool,
    pub quota: Option<i64>,
    pub used_quota: Option<i64>,
    pub credential_status: String,
    pub connection_status: String,
    pub sync_status: String,
    pub last_seen_at: String,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertMailbox {
    pub domain_id: i64,
    pub external_id: Option<String>,
    pub address: String,
    pub local_part: String,
    pub display_name: Option<String>,
    pub active: bool,
    pub quota: Option<i64>,
    pub used_quota: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct MailboxAssignment {
    pub id: i64,
    pub user_id: i64,
    pub mailbox_id: i64,
    pub can_view: bool,
    pub can_read: bool,
    pub can_reply: bool,
    pub can_send: bool,
    pub can_send_as: bool,
    pub can_mark_read: bool,
    pub can_move: bool,
    pub can_delete: bool,
    pub can_manage: bool,
    pub assigned_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct MailboxCredential {
    pub mailbox_id: i64,
    pub username: String,
    pub password_encrypted: String,
    pub credential_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_verified_at: Option<String>,
    pub verification_status: Option<String>,
    pub verification_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct MailboxSyncState {
    pub mailbox_id: i64,
    pub folder_name: String,
    pub folder_kind: Option<String>,
    pub uid_validity: i64,
    pub highest_uid: i64,
    pub highest_modseq: Option<i64>,
    pub last_incremental_sync_at: Option<String>,
    pub last_reconciliation_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_error: Option<String>,
    pub sync_status: String,
}
