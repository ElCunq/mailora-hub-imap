/// IMAP Connection Test Service
use anyhow::{Result, Context};
use async_imap::Session;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, TlsStream};
use tokio_util::compat::{TokioAsyncReadCompatExt, Compat};

use crate::models::account::Account;

type ImapSession = Session<Compat<TlsStream<TcpStream>>>;

/// Test IMAP connection with account credentials
pub async fn test_imap_connection(account: &Account) -> Result<ImapConnectionTestResult> {
    let (email, password) = account.get_credentials()
        .context("Failed to decode credentials")?;
    
    // Connect to IMAP
    let tcp_stream = TcpStream::connect((&account.imap_host as &str, account.imap_port))
        .await
        .context(format!("Failed to connect to {}:{}", account.imap_host, account.imap_port))?;
    
    let tls_connector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(false)
            .build()
            .context("Failed to build TLS connector")?
    );
    
    let tls_stream = tls_connector.connect(&account.imap_host, tcp_stream)
        .await
        .context("TLS handshake failed")?;
    
    let client = async_imap::Client::new(tls_stream.compat());
    
    // Login
    let mut session = client.login(&email, &password)
        .await
        .map_err(|e| anyhow::anyhow!("Login failed: {}", e.0))?;
    
    // Get capabilities
    let capabilities = session.capabilities()
        .await
        .context("Failed to get capabilities")?;
    
    let caps: Vec<String> = capabilities.iter()
        .map(|c| format!("{:?}", c))
        .collect();
    
    // List folders
    use futures::stream::StreamExt;
    let folders_stream = session.list(Some(""), Some("*"))
        .await
        .context("Failed to list folders")?;
    
    let folders: Vec<_> = folders_stream.collect::<Vec<_>>().await;
    let folder_names: Vec<String> = folders.iter()
        .filter_map(|f| f.as_ref().ok())
        .map(|f| f.name().to_string())
        .collect();
    
    // Select INBOX to get stats
    let inbox = session.select("INBOX")
        .await
        .context("Failed to select INBOX")?;
    
    let exists = inbox.exists;
    let recent = inbox.recent;
    let uidvalidity = inbox.uid_validity.unwrap_or(0);
    let uidnext = inbox.uid_next.unwrap_or(0);
    
    // Logout
    session.logout().await.ok();
    
    Ok(ImapConnectionTestResult {
        success: true,
        capabilities: caps,
        folders: folder_names,
        inbox_stats: InboxStats {
            exists,
            recent,
            uidvalidity,
            uidnext,
        },
    })
}

/// Fetch recent messages from INBOX
pub async fn fetch_recent_messages(account: &Account, limit: u32) -> Result<Vec<MessagePreview>> {
    let (email, password) = account.get_credentials()?;
    
    let tcp_stream = TcpStream::connect((&account.imap_host as &str, account.imap_port)).await?;
    let tls_connector = TlsConnector::from(native_tls::TlsConnector::builder().build()?);
    let tls_stream = tls_connector.connect(&account.imap_host, tcp_stream).await?;
    let client = async_imap::Client::new(tls_stream.compat());
    
    let mut session = client.login(&email, &password)
        .await
        .map_err(|e| anyhow::anyhow!("Login failed: {}", e.0))?;
    
    let inbox = session.select("INBOX").await?;
    let exists = inbox.exists;
    
    if exists == 0 {
        session.logout().await.ok();
        return Ok(vec![]);
    }
    
    // Fetch last N messages
    let start = if exists > limit { exists - limit + 1 } else { 1 };
    let range = format!("{}:{}", start, exists);
    
    use futures::stream::StreamExt;
    let messages_stream = session.fetch(&range, "(UID ENVELOPE FLAGS)")
        .await
        .context("Failed to fetch messages")?;
    
    let messages: Vec<_> = messages_stream.collect::<Vec<_>>().await;
    
    let mut previews = Vec::new();
    
    for msg_result in messages.iter() {
        if let Ok(msg) = msg_result {
            if let Some(envelope) = msg.envelope() {
            let subject = envelope.subject
                .as_ref()
                .and_then(|s| std::str::from_utf8(s).ok())
                .unwrap_or("<no subject>")
                .to_string();
            
            let from = envelope.from
                .as_ref()
                .and_then(|addrs| addrs.first())
                .and_then(|addr| {
                    let name = addr.name.as_ref().and_then(|n| std::str::from_utf8(n).ok());
                    let mailbox = addr.mailbox.as_ref().and_then(|m| std::str::from_utf8(m).ok());
                    let host = addr.host.as_ref().and_then(|h| std::str::from_utf8(h).ok());
                    
                    match (name, mailbox, host) {
                        (Some(n), Some(m), Some(h)) => Some(format!("{} <{}@{}>", n, m, h)),
                        (None, Some(m), Some(h)) => Some(format!("{}@{}", m, h)),
                        _ => None,
                    }
                })
                .unwrap_or_else(|| "<unknown>".to_string());
            
            let date = envelope.date
                .as_ref()
                .and_then(|d| std::str::from_utf8(d).ok())
                .map(|s| s.to_string());
            
            let flags: Vec<String> = msg.flags()
                .map(|f| format!("{:?}", f))
                .collect();
            
            previews.push(MessagePreview {
                uid: msg.uid.unwrap_or(0),
                subject,
                from,
                date,
                flags,
            });
        }
        }
    }
    
    session.logout().await.ok();
    
    Ok(previews)
}

#[derive(Debug, serde::Serialize)]
pub struct ImapConnectionTestResult {
    pub success: bool,
    pub capabilities: Vec<String>,
    pub folders: Vec<String>,
    pub inbox_stats: InboxStats,
}

#[derive(Debug, serde::Serialize)]
pub struct InboxStats {
    pub exists: u32,
    pub recent: u32,
    pub uidvalidity: u32,
    pub uidnext: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct MessagePreview {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: Option<String>,
    pub flags: Vec<String>,
}
