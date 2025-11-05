use lettre::transport::smtp::client::{TlsParameters, Tls};
use lettre::transport::smtp::authentication::Mechanism;
use tracing_subscriber::FmtSubscriber;
pub fn gmail_smtp_test() -> anyhow::Result<()> {
    // 0) Debug log aç (lettre diyaloğunu görmek için)
    let sub = FmtSubscriber::builder()
        .with_env_filter("info,lettre=trace")
        .finish();
    tracing::subscriber::set_global_default(sub).ok();

    // 1) Kimlik bilgilerini **temizle**
    let user_raw = std::env::var("GMAIL_USER")?;
    let pass_raw = std::env::var("GMAIL_APP_PASS")?;
    let user = user_raw.trim().to_string();
    let pass = pass_raw.split_whitespace().collect::<String>();
    println!(
        "user='{}' (len {})  app_pass_len={}",
        user,
        user.len(),
        pass.len()
    );

    let creds = Credentials::new(user.clone(), pass.clone());
    let tls = TlsParameters::builder("smtp.gmail.com".into()).build()?;

    // 2) STARTTLS + tek mekanizma (LOGIN)
    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com")?
        .hello_name(ClientId::Domain("mailora.local".into()))
        .tls(Tls::Required(tls))
        .authentication(vec![Mechanism::Login])
        .credentials(creds)
        .build();

    // 3) From == Gmail adresinle başla
    let email = Message::builder()
        .from(user.parse()?)
        .to(user.parse()?)
        .subject("gmail starttls test")
        .body(String::from("hi"))?;

    mailer.send(&email)?;
    Ok(())
}
use lettre::transport::smtp::extension::ClientId;
/// filepath: /mailora-hub-imap/mailora-hub-imap/src/smtp/mod.rs
use anyhow::Result;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

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

    pub fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), lettre::transport::smtp::Error> {
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

pub fn send_simple(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<()> {
    use lettre::{
        transport::smtp::{
            authentication::{Credentials, Mechanism},
            client::{Tls, TlsParameters},
        },
        Message, SmtpTransport, Transport,
    };
    use std::net::IpAddr;
    use std::time::Duration;

    // Trim whitespace that may sneak in from copied app passwords
    let clean_password: String = password.chars().filter(|c| !c.is_whitespace()).collect();
    let creds = Credentials::new(username.to_string(), clean_password);

    let tls = TlsParameters::builder(host.into())
        .dangerous_accept_invalid_certs(true)
        .build()?;

    let mut builder = match SmtpTransport::relay(host) {
        Ok(b) => b,
        Err(_) => SmtpTransport::builder_dangerous(host),
    };

    let client_id = std::env::var("SMTP_HELLO_NAME")
        .ok()
        .and_then(|val| match val.parse::<IpAddr>() {
            Ok(ip) => Some(ClientId::new(ip.to_string())),
            Err(_) => Some(ClientId::Domain(val)),
        })
        .unwrap_or_else(|| {
            host.parse::<IpAddr>()
                .map(|ip| ClientId::new(ip.to_string()))
                .unwrap_or_else(|_| ClientId::Domain(host.to_string()))
        });

    builder = builder
        .port(port)
        .hello_name(client_id)
        .authentication(vec![Mechanism::Plain, Mechanism::Login])
        .credentials(creds)
        .timeout(Some(Duration::from_secs(20)));

    let builder = if port == 465 {
        builder.tls(Tls::Wrapper(tls))
    } else {
        builder.tls(Tls::Required(tls))
    };

    let mailer = builder.build();

    let from_addr = username.parse()?;
    let to_addr = to.parse()?;
    let email = Message::builder()
        .from(from_addr)
        .to(to_addr)
        .subject(subject)
        .body(body.to_string())?;

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("SMTP gönderim başarısız: {:?}", e);
            Err(e.into())
        }
    }
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
    // Varsayılan olarak 587 portu kullanılır, istenirse parametre eklenebilir
    send_simple(host, 587, username, password, to, subject, body)?;

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
