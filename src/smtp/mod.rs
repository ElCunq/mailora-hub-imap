// filepath: /mailora-hub-imap/mailora-hub-imap/src/smtp/mod.rs
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::env;
use anyhow::Result;

pub struct SmtpClient {
    smtp_transport: SmtpTransport,
}

impl SmtpClient {
    pub fn new() -> Self {
        let smtp_server = env::var("SMTP_SERVER").expect("SMTP_SERVER must be set");
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

        let creds = Credentials::new(smtp_username, smtp_password);

        let smtp_transport = SmtpTransport::relay(&smtp_server)
            .expect("Failed to create SMTP relay")
            .credentials(creds)
            .build();

        SmtpClient { smtp_transport }
    }

    pub fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), lettre::transport::smtp::Error> {
        let email = Message::builder()
            .from("no-reply@example.com".parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body.to_string())
            .unwrap();

        self.smtp_transport.send(&email)?;
        Ok(())
    }
}

pub fn send_simple(host:&str, username:&str, password:&str, to:&str, subject:&str, body:&str) -> Result<()> {
    let creds = Credentials::new(username.to_string(), password.to_string());
    let mailer = SmtpTransport::relay(host)?.credentials(creds).build();
    let email = Message::builder()
        .from(username.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body.to_string())?;
    mailer.send(&email)?;
    Ok(())
}

/// Send email and log to events table
pub async fn send_and_log(
    pool: &sqlx::SqlitePool,
    mailbox: &str,
    actor: Option<&str>,
    host: &str,
    username: &str,
    password: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<()> {
    // Send email
    send_simple(host, username, password, to, subject, body)?;
    
    // Log OUT event
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    
    sqlx::query!(
        "INSERT INTO events (direction, mailbox, actor, peer, subject, ts) VALUES (?, ?, ?, ?, ?, ?)",
        "OUT",
        mailbox,
        actor,
        to,
        subject,
        ts
    )
    .execute(pool)
    .await?;
    
    tracing::info!(mailbox, to, subject, "Email sent and logged");
    Ok(())
}