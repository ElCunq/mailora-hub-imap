/// Message Body Fetch Service
use anyhow::{Context, Result};
use async_imap::Session;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, TlsStream};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

use crate::models::account::Account;

type ImapSession = Session<Compat<TlsStream<TcpStream>>>;

/// Fetch full message body with MIME parsing
pub async fn fetch_message_body(
    account: &Account,
    uid: u32,
    folder: Option<&str>,
) -> Result<MessageBody> {
    let (email, password) = account.get_credentials()?;
    let folder = folder.unwrap_or("INBOX");

    // Connect
    let tcp_stream = TcpStream::connect((&account.imap_host as &str, account.imap_port)).await?;
    let tls_connector = TlsConnector::from(native_tls::TlsConnector::builder().build()?);
    let tls_stream = tls_connector
        .connect(&account.imap_host, tcp_stream)
        .await?;
    let client = async_imap::Client::new(tls_stream.compat());

    let mut session = client
        .login(&email, &password)
        .await
        .map_err(|e| anyhow::anyhow!("Login failed: {}", e.0))?;

    // Select folder
    session
        .select(folder)
        .await
        .context(format!("Failed to select folder: {}", folder))?;

    // Fetch message
    let uid_str = uid.to_string();
    let mut messages = session
        .uid_fetch(&uid_str, "(UID ENVELOPE BODY.PEEK[] FLAGS)")
        .await
        .context("Failed to fetch message")?;

    use futures::StreamExt;
    let msg = messages
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("Message not found"))?
        .context("Failed to parse message")?;

    // Parse envelope
    let envelope = msg
        .envelope()
        .ok_or_else(|| anyhow::anyhow!("No envelope"))?;

    let subject = envelope
        .subject
        .as_ref()
        .and_then(|s| std::str::from_utf8(s.as_ref()).ok())
        .unwrap_or("<no subject>")
        .to_string();

    let from = envelope
        .from
        .as_ref()
        .and_then(|addrs| addrs.first())
        .and_then(|addr| {
            let name = addr
                .name
                .as_ref()
                .and_then(|n| std::str::from_utf8(n.as_ref()).ok());
            let mailbox = addr
                .mailbox
                .as_ref()
                .and_then(|m| std::str::from_utf8(m.as_ref()).ok());
            let host = addr
                .host
                .as_ref()
                .and_then(|h| std::str::from_utf8(h.as_ref()).ok());
            match (name, mailbox, host) {
                (Some(n), Some(m), Some(h)) => Some(format!("{} <{}@{}>", n, m, h)),
                (None, Some(m), Some(h)) => Some(format!("{}@{}", m, h)),
                _ => None,
            }
        })
        .unwrap_or_else(|| "<unknown>".to_string());

    let to = envelope
        .to
        .as_ref()
        .and_then(|addrs| addrs.first())
        .and_then(|addr| {
            let mailbox = addr
                .mailbox
                .as_ref()
                .and_then(|m| std::str::from_utf8(m.as_ref()).ok());
            let host = addr
                .host
                .as_ref()
                .and_then(|h| std::str::from_utf8(h.as_ref()).ok());
            match (mailbox, host) {
                (Some(m), Some(h)) => Some(format!("{}@{}", m, h)),
                _ => None,
            }
        })
        .unwrap_or_else(|| "<unknown>".to_string());

    let date = envelope
        .date
        .as_ref()
        .and_then(|d| std::str::from_utf8(d.as_ref()).ok())
        .map(|s| s.to_string());

    // Get body
    let body_bytes = msg.body().unwrap_or(&[]);
    let raw_body = String::from_utf8_lossy(body_bytes).to_string();

    // Parse MIME (basit version)
    let (plain_text, html_text, has_attachments) = parse_simple_mime(&raw_body);

    // Flags
    let flags: Vec<String> = msg.flags().map(|f| format!("{:?}", f)).collect();
    let raw_size = body_bytes.len();

    // Drop messages stream before using session again
    drop(messages);

    session.logout().await.ok();

    Ok(MessageBody {
        uid,
        folder: folder.to_string(),
        subject,
        from,
        to,
        date,
        plain_text,
        html_text,
        has_attachments,
        flags,
        raw_size: body_bytes.len(),
    })
}

fn parse_simple_mime(raw: &str) -> (Option<String>, Option<String>, bool) {
    let mut plain_text = None;
    let mut html_text = None;
    let mut has_attachments = false;

    // Basit MIME parsing - daha gelişmiş bir parser eklenebilir
    let lines: Vec<&str> = raw.lines().collect();
    let mut in_plain = false;
    let mut in_html = false;
    let mut plain_content = String::new();
    let mut html_content = String::new();

    for line in lines {
        // Content-Type detection
        if line.contains("Content-Type: text/plain") {
            in_plain = true;
            in_html = false;
        } else if line.contains("Content-Type: text/html") {
            in_html = true;
            in_plain = false;
        } else if line.contains("Content-Type:")
            && (line.contains("application/") || line.contains("image/"))
        {
            has_attachments = true;
            in_plain = false;
            in_html = false;
        } else if line.starts_with("--") && line.len() > 10 {
            // Boundary marker
            in_plain = false;
            in_html = false;
        } else if in_plain && !line.starts_with("Content-") && !line.is_empty() {
            plain_content.push_str(line);
            plain_content.push('\n');
        } else if in_html && !line.starts_with("Content-") && !line.is_empty() {
            html_content.push_str(line);
            html_content.push('\n');
        }
    }

    if !plain_content.is_empty() {
        plain_text = Some(plain_content);
    }
    if !html_content.is_empty() {
        html_text = Some(html_content);
    }

    // Fallback: Eğer MIME parsing başarısız olduysa tüm body'yi plain text olarak al
    if plain_text.is_none() && html_text.is_none() {
        plain_text = Some(raw.to_string());
    }

    (plain_text, html_text, has_attachments)
}

#[derive(Debug, Serialize)]
pub struct MessageBody {
    pub uid: u32,
    pub folder: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub date: Option<String>,
    pub plain_text: Option<String>,
    pub html_text: Option<String>,
    pub has_attachments: bool,
    pub flags: Vec<String>,
    pub raw_size: usize,
}
