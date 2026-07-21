use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    SuperAdmin,
    DomainAdmin,
    User,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SuperAdmin => "SuperAdmin",
            Self::DomainAdmin => "DomainAdmin",
            Self::User => "User",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "SuperAdmin" | "Admin" => Self::SuperAdmin,
            "DomainAdmin" => Self::DomainAdmin,
            _ => Self::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct MailoraUser {
    pub id: i64,
    pub email: Option<String>,
    pub username: Option<String>,
    #[serde(skip)]
    pub password_hash: String,
    pub role: String,
}

impl MailoraUser {
    pub fn get_email_or_username(&self) -> String {
        self.email
            .clone()
            .or_else(|| self.username.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password_hash: String,
    pub role: String,
}
