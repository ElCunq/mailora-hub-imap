import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/src/services/message_body_service.rs"
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

replacement = """    // Best-effort cache write (ignore errors e.g., when table missing)
    match sqlx::query("INSERT OR REPLACE INTO message_bodies (account_id, folder, uid, body, html_body, subject, from_addr, date, flags) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&account.id)
        .bind(folder)
        .bind(uid as i64)
        .bind(&body_text)
        .bind(&html_opt)
        .bind(&fetched.subject.unwrap_or_default())
        .bind(&fetched.from.unwrap_or_default())
        .bind(&fetched.date)
        .bind("[]")
        .execute(pool)
        .await {
            Ok(_) => tracing::info!("Inserted body cache for UID {}", uid),
            Err(e) => tracing::error!("Failed to insert body cache for UID {}: {:?}", uid, e),
        }"""

content = re.sub(
    r"    // Best-effort cache write \(ignore errors e\.g\., when table missing\).*?\.await;",
    replacement,
    content,
    flags=re.DOTALL
)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("Insert debug logs added!")
