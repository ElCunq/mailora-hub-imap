use mailora_hub_imap::db::run_migrations;
use mailora_hub_imap::imap::coordinator::SyncCoordinator;
use mailora_hub_imap::models::account::{Account, EmailProvider};
use mailora_hub_imap::services::message_sync_service::upsert_sent_message;
use sqlx::Row;

async fn setup_test_db() -> (sqlx::SqlitePool, Account) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool)
        .await
        .expect("migrations should run cleanly");

    let account = Account {
        id: "acc_1".to_string(),
        email: "test@domain.org".to_string(),
        provider: EmailProvider::Custom,
        display_name: Some("Test User".to_string()),
        imap_host: "imap.domain.org".to_string(),
        imap_port: 993,
        smtp_host: "smtp.domain.org".to_string(),
        smtp_port: 587,
        credentials_encrypted: "enc".to_string(),
        enabled: true,
        sync_frequency_secs: 300,
        last_sync_ts: Some(1000),
        created_at: 1000,
        updated_at: 1000,
        append_policy: None,
        sent_folder_hint: None,
        color: None,
        carddav_url: None,
        caldav_url: None,
        password: "pass".to_string(),
    };

    let _ = sqlx::query(
        "INSERT INTO accounts (id, email, provider, imap_host, imap_port, smtp_host, smtp_port, credentials_encrypted, created_at, updated_at)
         VALUES (?, ?, 'custom', ?, ?, ?, ?, 'enc', 1000, 1000)"
    )
    .bind(&account.id)
    .bind(&account.email)
    .bind(&account.imap_host)
    .bind(account.imap_port as i64)
    .bind(&account.smtp_host)
    .bind(account.smtp_port as i64)
    .execute(&pool)
    .await
    .unwrap();

    (pool, account)
}

#[tokio::test]
async fn test_atomic_upsert_and_boolean_flag_extraction() {
    let (pool, account) = setup_test_db().await;

    let folder_name = "INBOX";
    let folder_kind = SyncCoordinator::derive_folder_kind(folder_name);
    let flags_json = serde_json::to_string(&vec!["\\Recent", "\\Flagged"]).unwrap();
    let internal_date_ts = 1750000000i64;

    // 1. Upsert unread message into INBOX via atomic query pattern
    let res = sqlx::query(
        r#"INSERT INTO messages (
               account_id, folder, uid, message_id, subject, from_addr, to_addr, date,
               flags, is_seen, is_answered, is_flagged, is_deleted, is_draft, folder_kind, internal_date_ts,
               size, has_attachments, synced_at
           ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
           ON CONFLICT (account_id, folder, uid) DO UPDATE SET
               message_id = COALESCE(NULLIF(excluded.message_id, ''), messages.message_id),
               subject = COALESCE(NULLIF(excluded.subject, ''), messages.subject),
               from_addr = COALESCE(NULLIF(excluded.from_addr, ''), messages.from_addr),
               to_addr = COALESCE(NULLIF(excluded.to_addr, ''), messages.to_addr),
               date = COALESCE(NULLIF(excluded.date, ''), messages.date),
               flags = excluded.flags,
               is_seen = excluded.is_seen,
               is_answered = excluded.is_answered,
               is_flagged = excluded.is_flagged,
               is_deleted = excluded.is_deleted,
               is_draft = excluded.is_draft,
               folder_kind = COALESCE(excluded.folder_kind, messages.folder_kind),
               internal_date_ts = CASE WHEN excluded.internal_date_ts > 0 THEN excluded.internal_date_ts ELSE messages.internal_date_ts END,
               size = CASE WHEN excluded.size > 0 THEN excluded.size ELSE messages.size END,
               synced_at = datetime('now')"#
    )
    .bind(&account.id)
    .bind(folder_name)
    .bind(101i64)
    .bind("<msg101@test.org>")
    .bind("Hello World")
    .bind("Alice <alice@test.org>")
    .bind("test@domain.org")
    .bind("2026-07-21T10:00:00Z")
    .bind(&flags_json)
    .bind(false) // is_seen
    .bind(false) // is_answered
    .bind(true)  // is_flagged
    .bind(false) // is_deleted
    .bind(false) // is_draft
    .bind(&folder_kind)
    .bind(internal_date_ts)
    .bind(1024i64)
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

    assert!(res.rows_affected() > 0);

    // Verify boolean columns & folder_kind
    let row = sqlx::query("SELECT is_seen, is_flagged, is_answered, is_deleted, folder_kind, internal_date_ts FROM messages WHERE account_id = ? AND folder = ? AND uid = ?")
        .bind(&account.id)
        .bind("INBOX")
        .bind(101i64)
        .fetch_one(&pool)
        .await
        .unwrap();

    let is_seen: bool = row.get("is_seen");
    let is_flagged: bool = row.get("is_flagged");
    let is_answered: bool = row.get("is_answered");
    let is_deleted: bool = row.get("is_deleted");
    let fk: Option<String> = row.get("folder_kind");
    let ts: i64 = row.get("internal_date_ts");

    assert!(!is_seen, "message should be unread (is_seen = false)");
    assert!(is_flagged, "message should be flagged (is_flagged = true)");
    assert!(!is_answered);
    assert!(!is_deleted);
    assert_eq!(fk.as_deref(), Some("inbox"));
    assert_eq!(ts, 1750000000);

    // 2. Atomic UPSERT update of the same message (marking as Seen and Answered)
    let flags_json_upd = serde_json::to_string(&vec!["\\Seen", "\\Answered"]).unwrap();
    let res_upd = sqlx::query(
        r#"INSERT INTO messages (
               account_id, folder, uid, message_id, subject, from_addr, to_addr, date,
               flags, is_seen, is_answered, is_flagged, is_deleted, is_draft, folder_kind, internal_date_ts,
               size, has_attachments, synced_at
           ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
           ON CONFLICT (account_id, folder, uid) DO UPDATE SET
               message_id = COALESCE(NULLIF(excluded.message_id, ''), messages.message_id),
               subject = COALESCE(NULLIF(excluded.subject, ''), messages.subject),
               flags = excluded.flags,
               is_seen = excluded.is_seen,
               is_answered = excluded.is_answered,
               is_flagged = excluded.is_flagged,
               synced_at = datetime('now')"#
    )
    .bind(&account.id)
    .bind(folder_name)
    .bind(101i64)
    .bind("<msg101@test.org>")
    .bind("Hello World (Updated)")
    .bind("Alice <alice@test.org>")
    .bind("test@domain.org")
    .bind("2026-07-21T10:00:00Z")
    .bind(&flags_json_upd)
    .bind(true)  // is_seen
    .bind(true)  // is_answered
    .bind(false) // is_flagged
    .bind(false)
    .bind(false)
    .bind(&folder_kind)
    .bind(internal_date_ts)
    .bind(1024i64)
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

    assert!(res_upd.rows_affected() > 0);

    let row_updated = sqlx::query("SELECT is_seen, is_flagged, is_answered, subject FROM messages WHERE account_id = ? AND folder = ? AND uid = ?")
        .bind(&account.id)
        .bind("INBOX")
        .bind(101i64)
        .fetch_one(&pool)
        .await
        .unwrap();

    let is_seen_upd: bool = row_updated.get("is_seen");
    let is_flagged_upd: bool = row_updated.get("is_flagged");
    let is_answered_upd: bool = row_updated.get("is_answered");
    let subject_upd: String = row_updated.get("subject");

    assert!(is_seen_upd, "message should now be marked read (is_seen = true)");
    assert!(!is_flagged_upd, "message should no longer be flagged");
    assert!(is_answered_upd, "message should now be answered");
    assert_eq!(subject_upd, "Hello World (Updated)");
}

#[tokio::test]
async fn test_upsert_sent_message_and_folder_kind() {
    let (pool, account) = setup_test_db().await;

    upsert_sent_message(
        &pool,
        &account,
        "Sent",
        200,
        Some("Outgoing test"),
        Some("bob@test.org"),
    )
    .await
    .expect("upsert sent message should succeed");

    let row = sqlx::query("SELECT is_seen, folder_kind FROM messages WHERE account_id = ? AND folder = ? AND uid = ?")
        .bind(&account.id)
        .bind("Sent")
        .bind(200i64)
        .fetch_one(&pool)
        .await
        .unwrap();

    let is_seen: bool = row.get("is_seen");
    let folder_kind: Option<String> = row.get("folder_kind");

    assert!(is_seen, "sent message is automatically seen");
    assert_eq!(folder_kind.as_deref(), Some("sent"));
}

#[tokio::test]
async fn test_keyset_cursor_pagination_query() {
    let (pool, account) = setup_test_db().await;

    // Insert multiple messages across timestamps
    for i in 1..=5 {
        let ts = 1700000000 + i * 100;
        let _ = sqlx::query(
            "INSERT INTO messages (account_id, folder, uid, subject, internal_date_ts, is_seen, size, synced_at) VALUES (?, 'INBOX', ?, ?, ?, 0, 500, datetime('now'))"
        )
        .bind(&account.id)
        .bind(i as i64)
        .bind(format!("Msg {i}"))
        .bind(ts)
        .execute(&pool)
        .await
        .unwrap();
    }

    // Query top 2 messages ordered by internal_date_ts DESC, id DESC
    let rows_page1: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT id, internal_date_ts, uid FROM messages WHERE account_id = ? ORDER BY internal_date_ts DESC, id DESC LIMIT 2"
    )
    .bind(&account.id)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(rows_page1.len(), 2);
    assert_eq!(rows_page1[0].2, 5); // highest uid / timestamp
    assert_eq!(rows_page1[1].2, 4);

    let cursor_ts = rows_page1[1].1;
    let cursor_id = rows_page1[1].0;

    // Query next page using keyset/cursor filter: AND (internal_date_ts < ? OR (internal_date_ts = ? AND id < ?))
    let sql_page2 = format!(
        "SELECT id, internal_date_ts, uid FROM messages WHERE account_id = ? AND (internal_date_ts < {cursor_ts} OR (internal_date_ts = {cursor_ts} AND id < {cursor_id})) ORDER BY internal_date_ts DESC, id DESC LIMIT 2"
    );
    let rows_page2: Vec<(i64, i64, i64)> = sqlx::query_as(&sql_page2)
        .bind(&account.id)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(rows_page2.len(), 2);
    assert_eq!(rows_page2[0].2, 3);
    assert_eq!(rows_page2[1].2, 2);
}
