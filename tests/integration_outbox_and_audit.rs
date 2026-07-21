use mailora_hub_imap::audit::AuditService;
use mailora_hub_imap::db::run_migrations;
use mailora_hub_imap::domains::model::UpsertDomain;
use mailora_hub_imap::domains::DomainRepository;
use mailora_hub_imap::mailboxes::model::UpsertMailbox;
use mailora_hub_imap::mailboxes::MailboxRepository;
use mailora_hub_imap::mailcow::models::CreateMailcowInstance;
use mailora_hub_imap::mailcow::MailcowRepository;
use mailora_hub_imap::services::outbox_service::queue_email;
use mailora_hub_imap::users::model::CreateUserRequest;
use mailora_hub_imap::users::UserRepository;

#[tokio::test]
async fn test_outbox_queue_and_audit_logging() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.expect("migrations should apply cleanly");

    // Create user for FK constraint
    let u_repo = UserRepository::new(&pool);
    let user = u_repo
        .create(CreateUserRequest {
            email: "actor@example.com".into(),
            password_hash: "hash".into(),
            role: "User".into(),
        })
        .await
        .unwrap();

    // Create required Mailcow instance, domain, and mailbox for FK constraint
    let mc_repo = MailcowRepository::new(&pool);
    let instance = mc_repo
        .create(CreateMailcowInstance {
            name: "Test MC".into(),
            base_url: "https://mc.test".into(),
            api_key_encrypted: "key".into(),
            imap_host: "imap.test".into(),
            imap_port: 993,
            smtp_host: "smtp.test".into(),
            smtp_port: 587,
            enabled: Some(true),
        })
        .await
        .unwrap();

    let dom_repo = DomainRepository::new(&pool);
    let domain = dom_repo
        .upsert(UpsertDomain {
            mailcow_instance_id: instance.id,
            external_id: Some("dom1".into()),
            name: "example.com".into(),
            active: true,
            quota: None,
            mailbox_limit: None,
        })
        .await
        .unwrap();

    let mb_repo = MailboxRepository::new(&pool);
    let mailbox = mb_repo
        .upsert(UpsertMailbox {
            domain_id: domain.id,
            external_id: Some("mb1".into()),
            address: "sender@example.com".into(),
            local_part: "sender".into(),
            display_name: Some("Sender".into()),
            active: true,
            quota: None,
            used_quota: None,
        })
        .await
        .unwrap();

    // 1. Test outbox queue_email
    let outbox_id = queue_email(
        &pool,
        "1",
        "recipient@example.com",
        "Test Subject",
        "Test Body content",
    )
    .await
    .expect("email should queue successfully");

    assert!(!outbox_id.is_empty(), "queued email ID should not be empty");

    // Verify outbox record exists directly in db
    let status: String = sqlx::query_scalar("SELECT status FROM outbox WHERE id = ?")
        .bind(&outbox_id)
        .fetch_one(&pool)
        .await
        .expect("should find outbox row");
    assert_eq!(status, "queued");

    // 2. Test AuditService
    let audit_svc = AuditService::new(&pool);

    // Record general action
    let audit_id = audit_svc
        .record_action(
            Some(user.id),
            "TEST_ACTION",
            "outbox",
            Some(&outbox_id),
            Some(domain.id),
            Some(mailbox.id),
            Some("{\"test\": true}"),
            Some("127.0.0.1"),
            Some("TestAgent/1.0"),
        )
        .await
        .expect("should record audit action");
    assert!(audit_id > 0);

    // List logs and verify
    let logs = audit_svc
        .list_logs(Some(user.id), None, None, 10, 0)
        .await
        .expect("should list logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "TEST_ACTION");

    // 3. Test record_send_attempt and update_send_status
    let send_id = audit_svc
        .record_send_attempt(
            user.id,
            mailbox.id,
            "sender@example.com",
            "recipient@example.com",
            None,
            "pending",
        )
        .await
        .expect("should record send attempt");
    assert!(send_id > 0);

    audit_svc
        .update_send_status(send_id, "sent", Some("msg-id-12345"), Some("250 2.0.0 OK"))
        .await
        .expect("should update send status");

    let send_logs = audit_svc
        .list_send_audit(Some(mailbox.id), Some(user.id), 10, 0)
        .await
        .expect("should list send audit");
    assert_eq!(send_logs.len(), 1);
    assert_eq!(send_logs[0].status, "sent");
}
