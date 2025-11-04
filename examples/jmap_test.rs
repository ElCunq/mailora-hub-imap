// Stalwart JMAP client örnek kullanım
// Gereken: reqwest, serde, serde_json, anyhow

use mailora_hub_imap::jmap_client::JmapClient;
use tokio;

#[tokio::main]
async fn main() {
    let base_url = "http://localhost:8080"; // Stalwart sunucu adresi
    let api_token = "YOUR_API_TOKEN";
    let account_id = "YOUR_ACCOUNT_ID";
    let client = JmapClient::new(base_url, api_token);

    // JMAP session
    match client.get_session().await {
        Ok(session) => println!("Session: {:#?}", session),
        Err(e) => eprintln!("Session error: {}", e),
    }

    // Inbox mesajları
    match client.get_inbox_messages(account_id).await {
        Ok(messages) => println!("Inbox: {:#?}", messages),
        Err(e) => eprintln!("Inbox error: {}", e),
    }

    // Mesaj gönder
    match client
        .send_email(account_id, "to@example.com", "Test", "Merhaba JMAP!")
        .await
    {
        Ok(resp) => println!("Send: {:#?}", resp),
        Err(e) => eprintln!("Send error: {}", e),
    }
}
