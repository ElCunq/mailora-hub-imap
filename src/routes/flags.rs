use axum::{extract::{Path, State}, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct UpdateFlagsReq {
    pub seen: Option<bool>,
    pub flagged: Option<bool>,
    pub deleted: Option<bool>,
}

/// POST /messages/:account_id/:folder/:uid/flags
pub async fn update_flags(
    State(pool): State<SqlitePool>,
    Path((account_id, folder, uid)): Path<(String, String, u32)>,
    Json(req): Json<UpdateFlagsReq>,
) -> Json<serde_json::Value> {
    use crate::services::account_service;

    let account = match account_service::get_account(&pool, &account_id).await {
        Ok(Some(a)) => a,
        Ok(None) => return Json(json!({"ok": false, "error": "account not found"})),
        Err(e) => return Json(json!({"ok": false, "error": format!("db error: {}", e)})),
    };

    // Build IMAP STORE command
    let mut flags_cmds: Vec<&str> = Vec::new();
    if let Some(seen) = req.seen { flags_cmds.push(if seen { "+FLAGS (\\Seen)" } else { "-FLAGS (\\Seen)" }); }
    if let Some(flagged) = req.flagged { flags_cmds.push(if flagged { "+FLAGS (\\Flagged)" } else { "-FLAGS (\\Flagged)" }); }
    if let Some(deleted) = req.deleted { flags_cmds.push(if deleted { "+FLAGS (\\Deleted)" } else { "-FLAGS (\\Deleted)" }); }

    if flags_cmds.is_empty() {
        return Json(json!({"ok": false, "error": "no-op"}));
    }

    // Apply to IMAP
    let res = async {
        let mut imap = crate::imap::conn::connect(&account.imap_host, account.imap_port, &account.email, &account.password).await?;
        let session = &mut imap.session;
        session.select(&folder).await?;
        for cmd in flags_cmds.iter() {
            session.uid_store(&uid.to_string(), cmd).await?;
        }
        session.expunge().await.ok();
        let _ = session.logout().await;
        anyhow::Ok(())
    }.await;

    if let Err(e) = res {
        return Json(json!({"ok": false, "error": e.to_string()}));
    }

    // Update DB flags snapshot
    let flags_json = serde_json::to_string(&{
        let mut v = Vec::new();
        if req.seen.unwrap_or(false) { v.push("\\Seen"); }
        if req.flagged.unwrap_or(false) { v.push("\\Flagged"); }
        if req.deleted.unwrap_or(false) { v.push("\\Deleted"); }
        v
    }).unwrap_or("[]".into());

    let _ = sqlx::query(
        "UPDATE messages SET flags = ?, synced_at = datetime('now') WHERE account_id = ? AND folder = ? AND uid = ?",
    )
    .bind(flags_json)
    .bind(&account_id)
    .bind(&folder)
    .bind(uid as i64)
    .execute(&pool)
    .await;

    Json(json!({"ok": true}))
}
