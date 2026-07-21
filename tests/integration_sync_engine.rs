use mailora_hub_imap::db::run_migrations;
use mailora_hub_imap::imap::coordinator::SyncCoordinator;
use mailora_hub_imap::mailboxes::model::MailboxSyncState;
use mailora_hub_imap::mailboxes::{JobLockRepository, SyncRunRepository, SyncStateRepository};

async fn setup_dummy_mailbox(pool: &sqlx::SqlitePool, mailbox_id: i64) {
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO mailcow_instances (id, name, base_url, api_key_encrypted, imap_host, imap_port, smtp_host, smtp_port) VALUES (1, 'mc', 'https://mc', 'key', 'imap', 993, 'smtp', 587)"
    ).execute(pool).await;
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO domains (id, mailcow_instance_id, name, active) VALUES (1, 1, 'dom.org', 1)"
    ).execute(pool).await;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO accounts (id, email, provider, imap_host, imap_port, smtp_host, smtp_port, credentials_encrypted, created_at, updated_at) VALUES (?, 'user@dom.org', 'custom', 'imap', 993, 'smtp', 587, 'enc', 1000, 1000)"
    ).bind(mailbox_id.to_string()).execute(pool).await.expect("insert accounts failed");
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO mailboxes (id, domain_id, address, local_part, active) VALUES (?, 1, 'user@dom.org', 'user', 1)"
    ).bind(mailbox_id).execute(pool).await.expect("insert mailboxes failed");
}

#[tokio::test]
async fn test_sync_state_repository_and_uidvalidity_reset() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.expect("migrations should apply cleanly");
    setup_dummy_mailbox(&pool, 101).await;

    let repo = SyncStateRepository::new(&pool);

    // 1. Initial upsert
    let state = MailboxSyncState {
        mailbox_id: 101,
        folder_name: "INBOX".to_string(),
        folder_kind: Some("inbox".to_string()),
        uid_validity: 12345,
        highest_uid: 50,
        highest_modseq: Some(999),
        last_incremental_sync_at: None,
        last_reconciliation_at: None,
        last_success_at: None,
        last_error: None,
        sync_status: "idle".to_string(),
    };

    let saved = repo.upsert_sync_state(&state).await.expect("upsert failed");
    assert_eq!(saved.mailbox_id, 101);
    assert_eq!(saved.folder_name, "INBOX");
    assert_eq!(saved.uid_validity, 12345);
    assert_eq!(saved.highest_uid, 50);

    // 2. Update status and success
    repo.update_status(101, "INBOX", "syncing", None).await.unwrap();
    let fetched = repo.get_sync_state(101, "INBOX").await.unwrap().unwrap();
    assert_eq!(fetched.sync_status, "syncing");

    repo.update_success(101, "INBOX", 75, Some(1005), false).await.unwrap();
    let fetched_succ = repo.get_sync_state(101, "INBOX").await.unwrap().unwrap();
    assert_eq!(fetched_succ.highest_uid, 75);
    assert_eq!(fetched_succ.sync_status, "idle");
    assert!(fetched_succ.last_incremental_sync_at.is_some());

    // 3. Seed dummy message rows and test UIDVALIDITY reset
    sqlx::query(
        r#"INSERT INTO messages (account_id, folder, uid, subject, synced_at)
           VALUES ('101', 'INBOX', 1, 'Test 1', datetime('now')), ('101', 'INBOX', 2, 'Test 2', datetime('now'))"#
    )
    .execute(&pool)
    .await
    .unwrap();

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE account_id = '101' AND folder = 'INBOX'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);

    // Trigger UIDVALIDITY reset (server UIDVALIDITY changed to 99999)
    repo.handle_uidvalidity_reset(101, "INBOX", 99999).await.unwrap();

    // Verify messages for this folder were purged and state reset
    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE account_id = '101' AND folder = 'INBOX'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_after, 0);

    let reset_state = repo.get_sync_state(101, "INBOX").await.unwrap().unwrap();
    assert_eq!(reset_state.uid_validity, 99999);
    assert_eq!(reset_state.highest_uid, 0);
    assert_eq!(reset_state.last_error.as_deref(), Some("UIDVALIDITY reset"));
}

#[tokio::test]
async fn test_job_lock_repository_concurrency_control() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.expect("migrations should apply cleanly");

    let locks = JobLockRepository::new(&pool);

    // Acquire lock
    let acquired1 = locks.acquire_lock("mailbox_sync:1", "worker_1", 60).await.unwrap();
    assert!(acquired1, "First acquisition should succeed");

    // Second acquisition attempt should fail while lock is active
    let acquired2 = locks.acquire_lock("mailbox_sync:1", "worker_2", 60).await.unwrap();
    assert!(!acquired2, "Second acquisition should fail due to active lock");

    // Extend lock
    let extended = locks.extend_lock("mailbox_sync:1", "worker_1", 120).await.unwrap();
    assert!(extended, "Lock extension by owner should succeed");

    // Release lock
    locks.release_lock("mailbox_sync:1", "worker_1").await.unwrap();

    // Now another worker can acquire
    let acquired3 = locks.acquire_lock("mailbox_sync:1", "worker_2", 60).await.unwrap();
    assert!(acquired3, "Acquisition after release should succeed");
}

#[tokio::test]
async fn test_sync_run_repository_logging() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.expect("migrations should apply cleanly");
    setup_dummy_mailbox(&pool, 202).await;

    let runs = SyncRunRepository::new(&pool);
    let run_id = runs.start_run(202, "ALL", "incremental").await.unwrap();
    assert!(run_id > 0);

    runs.finish_run(run_id, "success", 10, 5, 1, 0, None).await.unwrap();

    let (status, new_cnt, upd_cnt, del_cnt): (String, Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT status, new_count, updated_count, deleted_count FROM sync_runs WHERE id = ?"
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(status, "success");
    assert_eq!(new_cnt, Some(10));
    assert_eq!(upd_cnt, Some(5));
    assert_eq!(del_cnt, Some(1));
}

#[test]
fn test_sync_coordinator_derive_folder_kind() {
    assert_eq!(SyncCoordinator::derive_folder_kind("INBOX"), Some("inbox".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Sent Items"), Some("sent".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Sent Mail"), Some("sent".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Drafts"), Some("drafts".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Deleted Items"), Some("trash".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Trash"), Some("trash".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Bin"), Some("trash".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Junk"), Some("junk".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Spam"), Some("junk".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("Archive"), Some("archive".to_string()));
    assert_eq!(SyncCoordinator::derive_folder_kind("CustomFolder"), None);
}

#[tokio::test]
async fn test_sync_coordinator_skips_when_job_locked() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.expect("migrations should apply cleanly");

    let locks = JobLockRepository::new(&pool);
    // Pre-lock the mailbox
    locks.acquire_lock("mailbox_sync:555", "other_worker", 300).await.unwrap();

    let coordinator = SyncCoordinator::new(pool.clone(), 5);
    let summary = coordinator.sync_mailbox(555, false).await.unwrap();

    assert_eq!(summary.mailbox_id, 555);
    assert_eq!(summary.skipped_reason.as_deref(), Some("Job already locked/running"));
}
