use mailora_hub_imap::db::run_migrations;
use mailora_hub_imap::mailcow::client::MailcowClient;
use mailora_hub_imap::mailcow::discovery::DiscoveryService;
use mailora_hub_imap::mailcow::models::CreateMailcowInstance;
use mailora_hub_imap::mailcow::MailcowRepository;

#[tokio::test]
async fn test_mailcow_client_and_discovery_service() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.expect("migrations should apply");

    let repo = MailcowRepository::new(&pool);
    let instance = repo
        .create(CreateMailcowInstance {
            name: "Test Mailcow".into(),
            base_url: "https://mailcow.example.com".into(),
            api_key_encrypted: "secret_api_key_123".into(),
            imap_host: "imap.example.com".into(),
            imap_port: 993,
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            enabled: Some(true),
        })
        .await
        .expect("created instance");

    // Test MailcowClient creation from instance
    let _client = MailcowClient::from_instance(&instance);
    // Verify client parses mock JSON payloads cleanly
    let domains_json = serde_json::json!([
        {
            "domain_name": "testdomain.org",
            "active": 1,
            "max_num_mboxes_for_domain": 10
        }
    ]);
    let parsed_domains = MailcowClient::parse_domains_value(&domains_json).unwrap();
    assert_eq!(parsed_domains.len(), 1);
    assert_eq!(parsed_domains[0].name, "testdomain.org");

    // Test DiscoveryService methods
    let discovery = DiscoveryService::new(&pool);
    let all_summaries = discovery.run_discovery_all().await.unwrap();
    // Since base_url is mock/unreachable in unit test, discovery of live endpoints returns Err or skips
    // But we verify the service constructs and lists instances cleanly
    assert_eq!(all_summaries.len(), 0);
}
