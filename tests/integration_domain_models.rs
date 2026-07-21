use mailora_hub_imap::db::run_migrations;
use mailora_hub_imap::domains::model::UpsertDomain;
use mailora_hub_imap::domains::DomainRepository;
use mailora_hub_imap::mailboxes::model::UpsertMailbox;
use mailora_hub_imap::mailboxes::MailboxRepository;
use mailora_hub_imap::mailcow::models::{CreateMailcowInstance, UpdateMailcowInstance};
use mailora_hub_imap::mailcow::MailcowRepository;
use mailora_hub_imap::users::model::CreateUserRequest;
use mailora_hub_imap::users::UserRepository;

#[tokio::test]
async fn test_domain_model_repositories_and_migrations() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Verify all migrations apply cleanly
    run_migrations(&pool).await.expect("migrations should apply cleanly");

    // 1. Test MailcowRepository
    let mailcow_repo = MailcowRepository::new(&pool);
    let instance = mailcow_repo
        .create(CreateMailcowInstance {
            name: "Mailcow Prod".into(),
            base_url: "https://mail.example.com".into(),
            api_key_encrypted: "enc_key_123".into(),
            imap_host: "imap.example.com".into(),
            imap_port: 993,
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            enabled: Some(true),
        })
        .await
        .expect("should create instance");

    assert_eq!(instance.name, "Mailcow Prod");
    assert_eq!(instance.base_url, "https://mail.example.com");

    let updated = mailcow_repo
        .update(
            instance.id,
            UpdateMailcowInstance {
                name: Some("Mailcow Prod Updated".into()),
                ..Default::default()
            },
        )
        .await
        .expect("should update instance");
    assert_eq!(updated.name, "Mailcow Prod Updated");

    mailcow_repo
        .update_discovery_status(instance.id, "success", None)
        .await
        .expect("should update status");
    let fetched = mailcow_repo.get_by_id(instance.id).await.unwrap();
    assert_eq!(fetched.last_discovery_status.as_deref(), Some("success"));

    // 2. Test DomainRepository
    let domain_repo = DomainRepository::new(&pool);
    let domain = domain_repo
        .upsert(UpsertDomain {
            mailcow_instance_id: instance.id,
            external_id: Some("mc_dom_1".into()),
            name: "example.org".into(),
            active: true,
            quota: Some(1024),
            mailbox_limit: Some(50),
        })
        .await
        .expect("should upsert domain");

    assert_eq!(domain.name, "example.org");
    assert_eq!(domain.quota, Some(1024));

    // Test upsert conflict resolution
    let domain_updated = domain_repo
        .upsert(UpsertDomain {
            mailcow_instance_id: instance.id,
            external_id: Some("mc_dom_1".into()),
            name: "example.org".into(),
            active: true,
            quota: Some(2048),
            mailbox_limit: Some(100),
        })
        .await
        .expect("should update on conflict");
    assert_eq!(domain_updated.id, domain.id);
    assert_eq!(domain_updated.quota, Some(2048));

    // 3. Test MailboxRepository
    let mailbox_repo = MailboxRepository::new(&pool);
    let mailbox = mailbox_repo
        .upsert(UpsertMailbox {
            domain_id: domain.id,
            external_id: Some("mc_mb_1".into()),
            address: "user@example.org".into(),
            local_part: "user".into(),
            display_name: Some("Test User".into()),
            active: true,
            quota: Some(500),
            used_quota: Some(100),
        })
        .await
        .expect("should upsert mailbox");

    assert_eq!(mailbox.address, "user@example.org");
    assert_eq!(mailbox.credential_status, "missing");

    // Save credentials
    let cred = mailbox_repo
        .save_credentials(mailbox.id, "user@example.org", "encrypted_pw_hash")
        .await
        .expect("should save credentials");
    assert_eq!(cred.username, "user@example.org");

    let mb_fetched = mailbox_repo.get_by_id(mailbox.id).await.unwrap();
    assert_eq!(mb_fetched.credential_status, "present");

    // 4. Test UserRepository and Assignments
    let user_repo = UserRepository::new(&pool);
    let user = user_repo
        .create(CreateUserRequest {
            email: "admin@example.org".into(),
            password_hash: "hash_pw".into(),
            role: "DomainAdmin".into(),
        })
        .await
        .expect("should create user");

    assert_eq!(user.role, "DomainAdmin");
    assert_eq!(user.get_email_or_username(), "admin@example.org");

    // Assign domain admin
    let dom_assign = domain_repo
        .assign_admin(user.id, domain.id, None)
        .await
        .expect("should assign domain admin");
    assert_eq!(dom_assign.domain_id, domain.id);

    let user_domains = domain_repo.list_user_domains(user.id).await.unwrap();
    assert_eq!(user_domains.len(), 1);
    assert_eq!(user_domains[0].name, "example.org");

    // Assign mailbox permissions
    let mb_assign = mailbox_repo
        .assign_user(
            user.id,
            mailbox.id,
            (true, true, true, true, false, true, true, false, true), // view, read, reply, send, send_as, mark_read, move, delete, manage
            None,
        )
        .await
        .expect("should assign mailbox");
    assert!(mb_assign.can_view && mb_assign.can_send && mb_assign.can_manage);
    assert!(!mb_assign.can_delete);

    let user_mailboxes = mailbox_repo.list_user_mailboxes(user.id).await.unwrap();
    assert_eq!(user_mailboxes.len(), 1);
    assert_eq!(user_mailboxes[0].0.address, "user@example.org");
    assert_eq!(user_mailboxes[0].1.can_reply, true);
}
