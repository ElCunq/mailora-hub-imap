import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/src/services/message_body_service.rs"
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

prefetch_fn = """
/// Prefetch bodies for the most recent N messages in a folder that are not yet in cache.
pub async fn prefetch_recent_bodies(account: &Account, folder: &str, limit: u32, pool: &SqlitePool) -> Result<()> {
    // Find up to `limit` recent messages in this folder that don't exist in message_bodies
    let uids = sqlx::query(
        "SELECT uid FROM messages 
         WHERE account_id = ? AND folder = ? 
         AND uid NOT IN (SELECT uid FROM message_bodies WHERE account_id = ? AND folder = ?)
         ORDER BY uid DESC LIMIT ?"
    )
    .bind(&account.id)
    .bind(folder)
    .bind(&account.id)
    .bind(folder)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    if uids.is_empty() {
        return Ok(());
    }
    
    tracing::info!("Prefetching {} missing bodies for {}/{}", uids.len(), account.email, folder);

    // Connect to IMAP
    let mut imap_session = crate::imap::conn::connect(
        &account.imap_host,
        account.imap_port,
        &account.email,
        &account.password,
    ).await?;

    imap_session.session.select(folder).await?;

    // We can fetch them individually or in chunks.
    // Fetching individually to not blow up memory, since full bodies can be large.
    for row in uids {
        let uid: i64 = sqlx::Row::get(&row, "uid");
        // Using existing helper but passing the already open session
        // Actually, we can just use the existing `fetch_message_body` but it opens a new connection.
        // Let's implement an optimized fetch that uses our session.
        if let Ok(mut stream) = imap_session.session.uid_fetch(uid.to_string(), "(UID BODY.PEEK[])").await {
            use futures::StreamExt;
            if let Some(Ok(fetch)) = stream.next().await {
                let body_bytes = fetch.body().or_else(|| fetch.section(b"")).unwrap_or(b"");
                if !body_bytes.is_empty() {
                    let mut subject = String::new();
                    let mut from = String::new();
                    let mut date_str = String::new();
                    
                    if let Some(parsed) = mail_parser::Message::parse(body_bytes) {
                        subject = parsed.subject().unwrap_or("").to_string();
                        match parsed.from() {
                            mail_parser::HeaderValue::Address(addr) => {
                                let name = addr.name.as_ref().map(|n| n.as_ref()).unwrap_or("");
                                let email = addr.address.as_ref().map(|a| a.as_ref()).unwrap_or("");
                                from = if !name.is_empty() { format!("{} <{}>", name, email) } else { email.to_string() };
                            }
                            mail_parser::HeaderValue::AddressList(list) => {
                                if let Some(addr) = list.first() {
                                    let name = addr.name.as_ref().map(|n| n.as_ref()).unwrap_or("");
                                    let email = addr.address.as_ref().map(|a| a.as_ref()).unwrap_or("");
                                    from = if !name.is_empty() { format!("{} <{}>", name, email) } else { email.to_string() };
                                }
                            }
                            _ => {}
                        }
                        if let Some(d) = parsed.date() {
                            date_str = d.to_rfc3339();
                        }
                        
                        let body_plain = parsed.body_text(0).unwrap_or("").to_string();
                        let body_html = parsed.body_html(0).map(|s| s.to_string());
                        
                        let _ = sqlx::query("INSERT OR IGNORE INTO message_bodies (account_id, folder, uid, body, html_body, subject, from_addr, date, flags) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                            .bind(&account.id)
                            .bind(folder)
                            .bind(uid)
                            .bind(&body_plain)
                            .bind(body_html)
                            .bind(&subject)
                            .bind(&from)
                            .bind(&date_str)
                            .bind("[]")
                            .execute(pool)
                            .await;
                    }
                }
            }
        }
    }
    
    let _ = imap_session.session.logout().await;
    Ok(())
}
"""

if "pub async fn prefetch_recent_bodies" not in content:
    content += "\n" + prefetch_fn

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)

print("Prefetch function added!")
