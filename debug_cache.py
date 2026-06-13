import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/src/services/message_body_service.rs"
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

replacement = """
    // Cache lookup (skip if force_refresh)
    if !force_refresh {
        match sqlx::query("SELECT body, html_body, subject, from_addr, date, flags FROM message_bodies WHERE account_id=? AND folder=? AND uid=?")
            .bind(&account.id)
            .bind(folder)
            .bind(uid as i64)
            .fetch_optional(pool)
            .await {
            Ok(Some(row)) => {
                tracing::info!("Cache HIT for UID {}", uid);
                match (row.try_get::<String,_>("body"), row.try_get::<Option<String>,_>("html_body")) {
                    (Ok(body), Ok(html_body_opt)) => {
                        let subject: String = row.try_get::<Option<String>,_>("subject").ok().flatten().unwrap_or_default();
                        let from: String = row.try_get::<Option<String>,_>("from_addr").ok().flatten().unwrap_or_default();
                        let date: Option<String> = row.try_get::<Option<String>,_>("date").ok().flatten();
                        let flags_json: String = row.try_get::<Option<String>,_>("flags").ok().flatten().unwrap_or_default();
                        let flags: Vec<String> = serde_json::from_str(&flags_json).unwrap_or_default();
                        return Ok(MessageBody { uid, folder: folder.to_string(), subject, from, date, flags, plain_text: body.clone(), html_text: html_body_opt, raw_size: body.len() });
                    }
                    (Err(e1), _) => tracing::warn!("Cache parse error body: {:?}", e1),
                    (_, Err(e2)) => tracing::warn!("Cache parse error html_body: {:?}", e2),
                }
            }
            Ok(None) => tracing::info!("Cache MISS for UID {}", uid),
            Err(e) => tracing::error!("Cache DB error: {:?}", e),
        }
    }
"""

content = re.sub(
    r"    // Cache lookup \(skip if force_refresh\).*?    }\n",
    replacement,
    content,
    flags=re.DOTALL
)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("Cache debug logs added!")
