use mailora_hub_imap::db::run_migrations;
use mailora_hub_imap::domains::model::UpsertDomain;
use mailora_hub_imap::domains::DomainRepository;
use mailora_hub_imap::mailboxes::model::UpsertMailbox;
use mailora_hub_imap::mailboxes::MailboxRepository;
use mailora_hub_imap::mailcow::models::CreateMailcowInstance;
use mailora_hub_imap::mailcow::MailcowRepository;
use mailora_hub_imap::rbac::authorization::{AuthorizationService, Permission};
use mailora_hub_imap::users::model::CreateUserRequest;
use mailora_hub_imap::users::UserRepository;

#[tokio::test]
async fn test_rbac_authorization_service_and_roles() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.expect("migrations should apply cleanly");

    // 1. Setup Mailcow Instance, Domain, and Mailboxes
    let mc_repo = MailcowRepository::new(&pool);
    let instance = mc_repo
        .create(CreateMailcowInstance {
            name: "Mailcow Core".into(),
            base_url: "https://mc.example.com".into(),
            api_key_encrypted: "enc_key".into(),
            imap_host: "imap.example.com".into(),
            imap_port: 993,
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            enabled: Some(true),
        })
        .await
        .unwrap();

    let dom_repo = DomainRepository::new(&pool);
    let domain1 = dom_repo
        .upsert(UpsertDomain {
            mailcow_instance_id: instance.id,
            external_id: Some("d1".into()),
            name: "corp.org".into(),
            active: true,
            quota: None,
            mailbox_limit: None,
        })
        .await
        .unwrap();

    let domain2 = dom_repo
        .upsert(UpsertDomain {
            mailcow_instance_id: instance.id,
            external_id: Some("d2".into()),
            name: "external.org".into(),
            active: true,
            quota: None,
            mailbox_limit: None,
        })
        .await
        .unwrap();

    let mb_repo = MailboxRepository::new(&pool);
    let mb_corp1 = mb_repo
        .upsert(UpsertMailbox {
            domain_id: domain1.id,
            external_id: Some("mb1".into()),
            address: "alice@corp.org".into(),
            local_part: "alice".into(),
            display_name: Some("Alice".into()),
            active: true,
            quota: None,
            used_quota: None,
        })
        .await
        .unwrap();

    let mb_corp2 = mb_repo
        .upsert(UpsertMailbox {
            domain_id: domain1.id,
            external_id: Some("mb2".into()),
            address: "bob@corp.org".into(),
            local_part: "bob".into(),
            display_name: Some("Bob".into()),
            active: true,
            quota: None,
            used_quota: None,
        })
        .await
        .unwrap();

    let mb_ext = mb_repo
        .upsert(UpsertMailbox {
            domain_id: domain2.id,
            external_id: Some("mb3".into()),
            address: "charlie@external.org".into(),
            local_part: "charlie".into(),
            display_name: Some("Charlie".into()),
            active: true,
            quota: None,
            used_quota: None,
        })
        .await
        .unwrap();

    // 2. Setup Users (`SuperAdmin`, `DomainAdmin`, `User`)
    let u_repo = UserRepository::new(&pool);
    let super_admin = u_repo
        .create(CreateUserRequest {
            email: "root@mailora.local".into(),
            password_hash: "hash".into(),
            role: "SuperAdmin".into(),
        })
        .await
        .unwrap();

    let domain_admin = u_repo
        .create(CreateUserRequest {
            email: "admin@corp.org".into(),
            password_hash: "hash".into(),
            role: "DomainAdmin".into(),
        })
        .await
        .unwrap();

    let regular_user = u_repo
        .create(CreateUserRequest {
            email: "user@corp.org".into(),
            password_hash: "hash".into(),
            role: "User".into(),
        })
        .await
        .unwrap();

    // Assign domain_admin to domain1
    dom_repo
        .assign_admin(domain_admin.id, domain1.id, Some(super_admin.id))
        .await
        .unwrap();

    // Assign regular_user to mb_corp1 with read + reply only (no delete, no send)
    mb_repo
        .assign_user(
            regular_user.id,
            mb_corp1.id,
            (true, true, true, false, false, true, false, false, false),
            Some(domain_admin.id),
        )
        .await
        .unwrap();

    // 3. Test `AuthorizationService` Checks
    let auth = AuthorizationService::new(&pool);

    // SuperAdmin checks
    assert!(auth.is_super_admin(super_admin.id).await.unwrap());
    assert!(!auth.is_super_admin(domain_admin.id).await.unwrap());
    assert!(auth.can_manage_domain(super_admin.id, domain1.id).await.unwrap());
    assert!(auth.can_manage_domain(super_admin.id, domain2.id).await.unwrap());
    let super_allowed = auth.get_allowed_mailbox_ids(super_admin.id).await.unwrap();
    assert_eq!(super_allowed.len(), 3); // Sees all active mailboxes

    // DomainAdmin checks
    assert!(auth.can_manage_domain(domain_admin.id, domain1.id).await.unwrap());
    assert!(!auth.can_manage_domain(domain_admin.id, domain2.id).await.unwrap());
    let da_allowed = auth.get_allowed_mailbox_ids(domain_admin.id).await.unwrap();
    assert_eq!(da_allowed, vec![mb_corp1.id, mb_corp2.id]); // Sees both mailboxes in domain1, not domain2
    let da_perms_mb1 = auth.get_mailbox_permissions(domain_admin.id, mb_corp1.id).await.unwrap();
    assert!(da_perms_mb1.can_view && da_perms_mb1.can_delete && da_perms_mb1.can_manage);

    // Regular User checks
    assert!(!auth.can_manage_domain(regular_user.id, domain1.id).await.unwrap());
    let u_allowed = auth.get_allowed_mailbox_ids(regular_user.id).await.unwrap();
    assert_eq!(u_allowed, vec![mb_corp1.id]); // Only sees mb_corp1
    let u_perms_mb1 = auth.get_mailbox_permissions(regular_user.id, mb_corp1.id).await.unwrap();
    assert!(u_perms_mb1.can_view && u_perms_mb1.can_read && u_perms_mb1.can_reply);
    assert!(!u_perms_mb1.can_send && !u_perms_mb1.can_delete && !u_perms_mb1.can_manage);

    assert!(auth.check_mailbox_permission(regular_user.id, mb_corp1.id, Permission::Read).await.unwrap());
    assert!(!auth.check_mailbox_permission(regular_user.id, mb_corp1.id, Permission::Delete).await.unwrap());

    // Check filter_viewable_mailboxes
    let test_ids = vec![mb_corp1.id, mb_corp2.id, mb_ext.id];
    let filtered_u = auth.filter_viewable_mailboxes(regular_user.id, &test_ids).await.unwrap();
    assert_eq!(filtered_u, vec![mb_corp1.id]);

    let filtered_da = auth.filter_viewable_mailboxes(domain_admin.id, &test_ids).await.unwrap();
    assert_eq!(filtered_da, vec![mb_corp1.id, mb_corp2.id]);
}
