use anyhow::Result;
/// Account models for multi-provider email integration
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EmailProvider {
    Gmail,
    Outlook,
    Yahoo,
    Icloud,
    #[default]
    Custom,
}

impl EmailProvider {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gmail" => Self::Gmail,
            "outlook" => Self::Outlook,
            "yahoo" => Self::Yahoo,
            "icloud" => Self::Icloud,
            _ => Self::Custom,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Gmail => "gmail",
            Self::Outlook => "outlook",
            Self::Yahoo => "yahoo",
            Self::Icloud => "icloud",
            Self::Custom => "custom",
        }
    }

    /// Get default IMAP/SMTP settings for known providers
    pub fn default_config(&self) -> ProviderConfig {
        match self {
            Self::Gmail => ProviderConfig {
                imap_host: "imap.gmail.com".to_string(),
                imap_port: 993,
                smtp_host: "smtp.gmail.com".to_string(),
                smtp_port: 587,
            },
            Self::Outlook => ProviderConfig {
                imap_host: "outlook.office365.com".to_string(),
                imap_port: 993,
                smtp_host: "smtp.office365.com".to_string(),
                smtp_port: 587,
            },
            Self::Yahoo => ProviderConfig {
                imap_host: "imap.mail.yahoo.com".to_string(),
                imap_port: 993,
                smtp_host: "smtp.mail.yahoo.com".to_string(),
                smtp_port: 587,
            },
            Self::Icloud => ProviderConfig {
                imap_host: "imap.mail.me.com".to_string(),
                imap_port: 993,
                smtp_host: "smtp.mail.me.com".to_string(),
                smtp_port: 587,
            },
            Self::Custom => ProviderConfig {
                imap_host: String::new(),
                imap_port: 993,
                smtp_host: String::new(),
                smtp_port: 587,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: String,
    pub email: String,
    #[sqlx(skip)]
    pub provider: EmailProvider,
    pub display_name: Option<String>,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    #[serde(skip_serializing)]
    pub credentials_encrypted: String, // Base64 encoded "email:password"
    pub enabled: bool,
    pub sync_frequency_secs: i64,
    pub last_sync_ts: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,

    // Helper field for password (populated from credentials_encrypted)
    #[sqlx(skip)]
    #[serde(skip)]
    pub password: String,
}

impl Account {
    /// Load account and decrypt password
    pub fn with_password(mut self) -> Result<Self> {
        let (_, password) = Self::decode_credentials(&self.credentials_encrypted)?;
        self.password = password;
        Ok(self)
    }
}

impl Account {
    /// Generate account ID from email
    pub fn generate_id(email: &str) -> String {
        format!("acc_{}", email.replace('@', "_").replace('.', "_"))
    }

    /// Encode credentials (simple base64, upgrade to OS keychain later)
    pub fn encode_credentials(email: &str, password: &str) -> String {
        use base64::Engine;
        let creds = format!("{}:{}", email, password);
        base64::engine::general_purpose::STANDARD.encode(creds.as_bytes())
    }

    /// Decode credentials
    pub fn decode_credentials(encoded: &str) -> Result<(String, String)> {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(encoded)?;
        let creds = String::from_utf8(decoded)?;
        let parts: Vec<&str> = creds.splitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid credentials format");
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Get credentials for this account
    pub fn get_credentials(&self) -> Result<(String, String)> {
        Self::decode_credentials(&self.credentials_encrypted)
    }
}

// Database mapping helpers
impl Account {
    pub fn provider_str(&self) -> String {
        self.provider.as_str().to_string()
    }
}
