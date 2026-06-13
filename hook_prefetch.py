import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/src/services/message_sync_service.rs"
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

replacement = """
    // === BACKGROUND PREFETCH ===
    // Spawns a background task to fetch full bodies for the 20 most recent emails
    let prefetch_pool = pool.clone();
    let prefetch_account = account.clone();
    let prefetch_folder = folder.to_string();
    tokio::spawn(async move {
        if let Err(e) = crate::services::message_body_service::prefetch_recent_bodies(
            &prefetch_account, 
            &prefetch_folder, 
            20, 
            &prefetch_pool
        ).await {
            tracing::error!("Prefetch error for {}/{}: {}", prefetch_account.email, prefetch_folder, e);
        }
    });

    Ok(SyncStats {"""

if "// === BACKGROUND PREFETCH ===" not in content:
    content = content.replace("    Ok(SyncStats {", replacement)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("Prefetch hooked into sync!")
