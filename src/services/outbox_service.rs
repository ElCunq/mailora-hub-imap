use crate::models::{account::Account, outbox::OutboxEmail};
use crate::services::account_service;
use anyhow::Context;
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::time::sleep;

/// Queue an email to be sent
pub async fn queue_email(
    pool: &SqlitePool,
    account_id: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO outbox (id, account_id, to_addr, subject, body, status, retries, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, 'queued', 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(account_id)
    .bind(to)
    .bind(subject)
    .bind(body)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Background loop to process outbox with exponential backoff and permission enforcement
pub async fn start_outbox_loop(pool: SqlitePool) {
    tracing::info!("Starting Outbox Service loop...");
    loop {
        if let Err(e) = process_batch(&pool).await {
            tracing::error!("Outbox processing error: {}", e);
        }
        sleep(Duration::from_secs(10)).await;
    }
}

async fn process_batch(pool: &SqlitePool) -> Result<(), anyhow::Error> {
    // Select queued or failed (with retries < 5 and exponential backoff: 30s, 60s, 120s, 240s)
    let emails = sqlx::query_as::<_, OutboxEmail>(
        "SELECT id, account_id, to_addr, subject, body, status, retries, last_error, 
         CAST(strftime('%s', created_at) AS TEXT) as created_at, 
         CAST(strftime('%s', updated_at) AS TEXT) as updated_at
         FROM outbox 
         WHERE status = 'queued' OR (
             status = 'failed' 
             AND retries < 5 
             AND (CAST(strftime('%s', 'now') AS INTEGER) - CAST(strftime('%s', updated_at) AS INTEGER)) >= ((1 << MIN(retries, 8)) * 15)
         )
         ORDER BY created_at ASC
         LIMIT 10"
    )
    .fetch_all(pool)
    .await?;

    if emails.is_empty() {
        return Ok(());
    }

    tracing::info!("Outbox: Processing {} emails", emails.len());

    for email in emails {
        // Mark as processing
        let _ = sqlx::query("UPDATE outbox SET status = 'processing', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&email.id)
            .execute(pool)
            .await;

        // Try to check if account_id is a v3 numeric mailbox_id first
        let is_numeric = email.account_id.parse::<i64>().is_ok();
        if is_numeric {
            let mailbox_id = email.account_id.parse::<i64>().unwrap();
            let mb_repo = crate::mailboxes::MailboxRepository::new(pool);
            match mb_repo.get_by_id(mailbox_id).await {
                Ok(mb) => {
                    // Check if active and user/permissions are valid before sending
                    if !mb.active {
                        let err_msg = "Mailbox is inactive or disabled";
                        tracing::error!("Outbox: {} for email {}", err_msg, email.id);
                        let _ = sqlx::query(
                            "UPDATE outbox SET status = 'failed', retries = retries + 1, last_error = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                        )
                        .bind(err_msg)
                        .bind(&email.id)
                        .execute(pool)
                        .await;
                        continue;
                    }

                    match send_via_v3_mailbox(pool, &mb, &email.to_addr, &email.subject, &email.body).await {
                        Ok((message_id, smtp_response)) => {
                            tracing::info!("Outbox: Email {} sent successfully via mailbox {}", email.id, mailbox_id);
                            let _ = sqlx::query("UPDATE outbox SET status = 'sent', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                                .bind(&email.id)
                                .execute(pool)
                                .await;
                            let audit_svc = crate::audit::AuditService::new(pool);
                            let _ = audit_svc.record_send_attempt(0, mailbox_id, &mb.address, &email.to_addr, Some(&message_id), "sent").await;
                            let _ = audit_svc.record_action(None, "send_email", "outbox", Some(&email.id), Some(mb.domain_id), Some(mailbox_id), Some(&smtp_response), None, None).await;
                        }
                        Err(e) => {
                            tracing::error!("Outbox: Failed to send {} via mailbox {}: {}", email.id, mailbox_id, e);
                            let _ = sqlx::query(
                                "UPDATE outbox SET status = 'failed', retries = retries + 1, last_error = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                            )
                            .bind(e.to_string())
                            .bind(&email.id)
                            .execute(pool)
                            .await;
                            let audit_svc = crate::audit::AuditService::new(pool);
                            let _ = audit_svc.record_send_attempt(0, mailbox_id, &mb.address, &email.to_addr, None, "failed").await;
                        }
                    }
                    continue;
                }
                Err(_) => {
                    // Fall through to legacy account check below
                }
            }
        }

        // Validate account via legacy account_service
        let account_opt = account_service::get_account(pool, &email.account_id).await?;
        match account_opt {
            Some(account) => {
                match send_via_smtp(&account, &email.to_addr, &email.subject, &email.body).await {
                    Ok(message_id) => {
                        tracing::info!("Outbox: Email {} sent successfully", email.id);
                        let _ = sqlx::query("UPDATE outbox SET status = 'sent', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&email.id)
                            .execute(pool)
                            .await;
                        let audit_svc = crate::audit::AuditService::new(pool);
                        let _ = audit_svc.record_send_attempt(0, 0, &account.email, &email.to_addr, Some(&message_id), "sent").await;
                    }
                    Err(e) => {
                        tracing::error!("Outbox: Failed to send {}: {}", email.id, e);
                        let _ = sqlx::query(
                            "UPDATE outbox SET status = 'failed', retries = retries + 1, last_error = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                        )
                        .bind(e.to_string())
                        .bind(&email.id)
                        .execute(pool)
                        .await;
                        let audit_svc = crate::audit::AuditService::new(pool);
                        let _ = audit_svc.record_send_attempt(0, 0, &account.email, &email.to_addr, None, "failed").await;
                    }
                }
            }
            None => {
                tracing::error!("Outbox: Account {} not found for email {}", email.account_id, email.id);
                let _ = sqlx::query("UPDATE outbox SET status = 'failed', last_error = 'Account not found', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                    .bind(&email.id)
                    .execute(pool)
                    .await;
            }
        }
    }

    Ok(())
}

async fn send_via_v3_mailbox(
    pool: &SqlitePool,
    mailbox: &crate::mailboxes::Mailbox,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(String, String), anyhow::Error> {
    use crate::mailboxes::MailboxRepository;
    use crate::domains::DomainRepository;
    use crate::mailcow::MailcowRepository;
    use crate::services::crypto::decrypt_secret;
    use lettre::{
        transport::smtp::authentication::Credentials,
        transport::smtp::client::Tls,
        AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    };

    let mb_repo = MailboxRepository::new(pool);
    let cred = mb_repo.get_credentials(mailbox.id).await.context("Mailbox credentials missing")?;
    let decrypted_pass = decrypt_secret(&cred.password_encrypted).context("Failed to decrypt password")?;

    let domain_repo = DomainRepository::new(pool);
    let domain = domain_repo.get_by_id(mailbox.domain_id).await.context("Domain not found")?;

    let mailcow_repo = MailcowRepository::new(pool);
    let instance = mailcow_repo.get_by_id(domain.mailcow_instance_id).await.context("Mailcow instance not found")?;

    let creds = Credentials::new(mailbox.address.clone(), decrypted_pass);

    let tls = if instance.smtp_port == 465 {
        Tls::Wrapper(lettre::transport::smtp::client::TlsParameters::new(instance.smtp_host.clone())?)
    } else {
        Tls::Required(lettre::transport::smtp::client::TlsParameters::new(instance.smtp_host.clone())?)
    };

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&instance.smtp_host)?
        .credentials(creds)
        .port(instance.smtp_port as u16)
        .tls(tls)
        .build();

    let (email_msg, msg_id) = crate::smtp::build_email(&mailbox.address, to, subject, body)?;

    let mailer_result = tokio::time::timeout(std::time::Duration::from_secs(20), mailer.send(email_msg)).await;
    match mailer_result {
        Ok(result) => {
            let resp = result?;
            Ok((msg_id, format!("{:?}", resp)))
        }
        Err(_) => Err(anyhow::anyhow!("SMTP connection timed out after 20 seconds")),
    }
}

async fn send_via_smtp(account: &Account, to: &str, subject: &str, body: &str) -> Result<String, anyhow::Error> {
    use lettre::{
        transport::smtp::authentication::Credentials,
        transport::smtp::client::Tls,
        AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    };

    let creds = Credentials::new(account.email.clone(), account.password.clone());
    
    let tls = if account.smtp_port == 465 {
        Tls::Wrapper(lettre::transport::smtp::client::TlsParameters::new(
            account.smtp_host.clone(),
        )?)
    } else {
        Tls::Required(lettre::transport::smtp::client::TlsParameters::new(
            account.smtp_host.clone(),
        )?)
    };

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&account.smtp_host)?
        .credentials(creds)
        .port(account.smtp_port)
        .tls(tls)
        .build();

    let (email_msg, msg_id) = crate::smtp::build_email(&account.email, to, subject, body)?;

    let mailer_result = tokio::time::timeout(std::time::Duration::from_secs(15), mailer.send(email_msg)).await;
    match mailer_result {
        Ok(result) => {
            result?;
            Ok(msg_id)
        }
        Err(_) => {
            Err(anyhow::anyhow!("SMTP connection timed out after 15 seconds"))
        }
    }
}
