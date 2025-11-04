use anyhow::Result;
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use crate::imap::folders::list_mailboxes;
use crate::persist::ACCOUNT_STATE;
use crate::services::diff_service::ChangeItem;
use crate::services::diff_service::ACCOUNTS;
use crate::services::diff_service::{
    decode_cursor, encode_cursor, CursorToken, DiffResponseWire, FolderCursor,
};

#[derive(Deserialize)]
pub struct DiffQs {
    pub accountId: String,
    pub since: Option<String>,
    pub folder: Option<String>,
}

#[derive(Deserialize)]
pub struct BodyQs {
    pub accountId: String,
    pub uid: u32,
    pub folder: Option<String>,
}

#[derive(Serialize)]
pub struct BodyResponse {
    pub accountId: String,
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: Option<String>,
    pub size: Option<u32>,
    pub flags: Vec<String>,
    pub body: String,
}

#[derive(Deserialize)]
pub struct FoldersQs {
    pub accountId: String,
}

#[derive(Deserialize)]
pub struct AttachQs {
    pub accountId: String,
    pub uid: u32,
}

async fn list_all_folders_excluding_spam(
    creds: &crate::services::diff_service::AccountCreds,
) -> Result<Vec<String>, (axum::http::StatusCode, String)> {
    let folders = list_mailboxes(&creds.host, creds.port, &creds.email, &creds.password)
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;
    let names: Vec<String> = folders
        .into_iter()
        .map(|f| f.name)
        .filter(|n| {
            let l = n.to_lowercase();
            !(l.contains("spam") || l.contains("junk") || l.contains("çöp"))
        })
        .collect();
    Ok(names)
}

pub async fn diff_handler(
    Query(q): Query<DiffQs>,
) -> Result<Json<DiffResponseWire>, (axum::http::StatusCode, String)> {
    let account_id = q.accountId.clone();
    let creds_opt = {
        let store = ACCOUNTS.read().await;
        store.get(&account_id).cloned()
    };
    let creds = creds_opt.ok_or((
        axum::http::StatusCode::NOT_FOUND,
        "account not logged in".into(),
    ))?;

    if let Some(tok) = q.since.as_ref() {
        let cur =
            decode_cursor(tok).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;
        let multi = cur.folders.clone();
        let target_folders: Vec<String> = if let Some(fset) = multi {
            fset.into_iter().map(|f| f.name).collect()
        } else if cur.folder == "*" {
            list_all_folders_excluding_spam(&creds).await?
        } else {
            vec![q.folder.clone().unwrap_or(cur.folder.clone())]
        };
        let mut changes: Vec<ChangeItem> = Vec::new();
        let mut next_cur = cur.clone();
        let mut new_last_overall = cur.last_uid;
        // Track per-folder last_uid updates
        let mut per_folder_updates: Vec<(String, u32)> = Vec::new();
        for folder in target_folders.iter() {
            let last_for_folder = cur
                .folders
                .as_ref()
                .and_then(|fs| fs.iter().find(|f| &f.name == folder).map(|f| f.last_uid))
                .unwrap_or(cur.last_uid);
            let (mut new_last_f, mut new_msgs) = crate::imap::sync::fetch_new_since_in(
                &creds.host,
                creds.port,
                &creds.email,
                &creds.password,
                last_for_folder,
                folder,
            )
            .await
            .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;
            if new_msgs.is_empty() && folder == "INBOX" {
                if let Ok(state_vec_probe) = crate::imap::sync::snapshot_state(
                    &creds.host,
                    creds.port,
                    &creds.email,
                    &creds.password,
                )
                .await
                {
                    let newer_uids: Vec<u32> = state_vec_probe
                        .iter()
                        .filter_map(|s| {
                            if s.uid > last_for_folder {
                                Some(s.uid)
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !newer_uids.is_empty() {
                        match crate::imap::sync::fetch_meta_for_uids_in(
                            &creds.host,
                            creds.port,
                            &creds.email,
                            &creds.password,
                            &newer_uids,
                            folder,
                        )
                        .await
                        {
                            Ok(fetched) if !fetched.is_empty() => {
                                new_msgs = fetched;
                            }
                            _ => {
                                new_msgs = newer_uids
                                    .into_iter()
                                    .map(|uid| crate::imap::sync::NewMessageMeta {
                                        uid,
                                        subject: String::new(),
                                        from: String::new(),
                                        date: None,
                                        size: None,
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
            // Add change items
            for m in &new_msgs {
                changes.push(ChangeItem::MessageAdded {
                    folder: folder.clone(),
                    uid: m.uid,
                    subject: m.subject.clone(),
                    from: m.from.clone(),
                    date: m.date.clone(),
                    size: m.size,
                });
            }
            // Compute per-folder new_last using either the returned new_last_f or max uid from new_msgs
            if !new_msgs.is_empty() {
                if let Some(max_uid) = new_msgs.iter().map(|m| m.uid).max() {
                    if max_uid > new_last_f {
                        new_last_f = max_uid;
                    }
                }
            }
            if new_last_f < last_for_folder {
                new_last_f = last_for_folder;
            }
            per_folder_updates.push((folder.clone(), new_last_f));
            if new_last_f > new_last_overall {
                new_last_overall = new_last_f;
            }
        }
        next_cur.folder = if cur.folders.is_some() {
            "*".into()
        } else {
            cur.folder.clone()
        };
        next_cur.last_uid = new_last_overall;
        if let Some(fset) = cur.folders.as_ref() {
            let mut updated: Vec<FolderCursor> = Vec::new();
            for fc in fset.iter() {
                if let Some((_, nl)) = per_folder_updates.iter().find(|(name, _)| name == &fc.name)
                {
                    updated.push(FolderCursor {
                        name: fc.name.clone(),
                        uidvalidity: fc.uidvalidity,
                        last_uid: *nl,
                    });
                } else {
                    updated.push(FolderCursor {
                        name: fc.name.clone(),
                        uidvalidity: fc.uidvalidity,
                        last_uid: fc.last_uid,
                    });
                }
            }
            next_cur.folders = Some(updated);
        }
        let next = encode_cursor(&next_cur);
        let since = encode_cursor(&cur);
        return Ok(Json(DiffResponseWire {
            accountId: account_id,
            since,
            next,
            changes,
        }));
    }

    // Initial: build multi-folder cursor aggregating all (except Spam)
    let names = list_all_folders_excluding_spam(&creds).await?;
    let mut fcs: Vec<FolderCursor> = Vec::new();
    for name in names.iter() {
        if let Ok(snap) = crate::imap::sync::initial_snapshot_in(
            &creds.host,
            creds.port,
            &creds.email,
            &creds.password,
            name,
        )
        .await
        {
            fcs.push(FolderCursor {
                name: name.clone(),
                uidvalidity: snap.uidvalidity,
                last_uid: snap.last_uid,
            });
        }
    }
    let base_last = fcs.iter().map(|f| f.last_uid).max().unwrap_or(0);
    let cur = CursorToken {
        folder: "*".into(),
        uidvalidity: 0,
        last_uid: base_last,
        modseq: None,
        folders: Some(fcs),
    };
    let token = encode_cursor(&cur);
    Ok(Json(DiffResponseWire {
        accountId: account_id,
        since: token.clone(),
        next: token,
        changes: vec![],
    }))
}

pub async fn body_handler(
    Query(q): Query<BodyQs>,
) -> Result<Json<BodyResponse>, (axum::http::StatusCode, String)> {
    let creds_opt = {
        let store = crate::services::diff_service::ACCOUNTS.read().await;
        store.get(&q.accountId).cloned()
    };
    let creds = creds_opt.ok_or((
        axum::http::StatusCode::NOT_FOUND,
        "account not logged in".into(),
    ))?;

    if let Some(folder) = q.folder.as_ref() {
        tracing::debug!(accountId=%q.accountId, %folder, uid=q.uid, "/body: direct folder fetch");
        match crate::imap::sync::fetch_message_body_in(
            &creds.host,
            creds.port,
            &creds.email,
            &creds.password,
            q.uid,
            folder,
        )
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?
        {
            Some(m) => {
                return Ok(Json(BodyResponse {
                    accountId: q.accountId,
                    uid: m.uid,
                    subject: m.subject,
                    from: m.from,
                    date: m.date,
                    size: m.size,
                    flags: m.flags,
                    body: m.body,
                }))
            }
            None => {
                return Err((
                    axum::http::StatusCode::NOT_FOUND,
                    "message not found".into(),
                ))
            }
        }
    }

    // Try INBOX first
    tracing::debug!(accountId=%q.accountId, uid=q.uid, "/body: try INBOX first");
    if let Ok(Some(m)) = crate::imap::sync::fetch_message_body_in(
        &creds.host,
        creds.port,
        &creds.email,
        &creds.password,
        q.uid,
        "INBOX",
    )
    .await
    {
        return Ok(Json(BodyResponse {
            accountId: q.accountId.clone(),
            uid: m.uid,
            subject: m.subject,
            from: m.from,
            date: m.date,
            size: m.size,
            flags: m.flags,
            body: m.body,
        }));
    }

    // Scan non-spam folders
    let folders = list_all_folders_excluding_spam(&creds).await?;
    for name in folders {
        if name == "INBOX" {
            continue;
        }
        tracing::debug!(accountId=%q.accountId, folder=%name, uid=q.uid, "/body: scanning folder");
        if let Ok(Some(m)) = crate::imap::sync::fetch_message_body_in(
            &creds.host,
            creds.port,
            &creds.email,
            &creds.password,
            q.uid,
            &name,
        )
        .await
        {
            return Ok(Json(BodyResponse {
                accountId: q.accountId.clone(),
                uid: m.uid,
                subject: m.subject,
                from: m.from,
                date: m.date,
                size: m.size,
                flags: m.flags,
                body: m.body,
            }));
        }
    }

    tracing::debug!(accountId=%q.accountId, uid=q.uid, "/body: not found in any folder");
    Err((
        axum::http::StatusCode::NOT_FOUND,
        "message not found".into(),
    ))
}

pub async fn folders_handler(
    Query(q): Query<FoldersQs>,
) -> Result<Json<Vec<crate::imap::folders::FolderInfo>>, (axum::http::StatusCode, String)> {
    let creds_opt = {
        let store = ACCOUNTS.read().await;
        store.get(&q.accountId).cloned()
    };
    let creds = creds_opt.ok_or((
        axum::http::StatusCode::NOT_FOUND,
        "account not logged in".into(),
    ))?;
    let folders = crate::imap::folders::list_mailboxes(
        &creds.host,
        creds.port,
        &creds.email,
        &creds.password,
    )
    .await
    .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(folders))
}

pub async fn attachments_handler(
    Query(q): Query<AttachQs>,
) -> Result<Json<Vec<crate::imap::sync::AttachmentMeta>>, (axum::http::StatusCode, String)> {
    let creds_opt = {
        let store = ACCOUNTS.read().await;
        store.get(&q.accountId).cloned()
    };
    let creds = creds_opt.ok_or((
        axum::http::StatusCode::NOT_FOUND,
        "account not logged in".into(),
    ))?;
    let atts = crate::imap::sync::list_attachments(
        &creds.host,
        creds.port,
        &creds.email,
        &creds.password,
        q.uid,
    )
    .await
    .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(atts))
}

fn int_err<E: std::fmt::Display>(e: E) -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
