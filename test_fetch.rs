use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tls = async_native_tls::TlsConnector::new();
    let client = async_imap::connect(("imap.gmail.com", 993), "imap.gmail.com", tls).await?;
    let mut session = client.login("cenkorfa1@gmail.com", "cenk3003!").await.unwrap();
    session.select("INBOX").await?;
    
    // Fetch last message
    let mut stream = session.fetch("1:1", "(UID FLAGS INTERNALDATE RFC822.SIZE BODY.PEEK[])").await?;
    use futures::StreamExt;
    if let Some(msg) = stream.next().await {
        let fetch = msg?;
        println!("UID: {:?}", fetch.uid);
        println!("header: {:?}", fetch.header().map(|b| b.len()));
        println!("body: {:?}", fetch.body().map(|b| b.len()));
        println!("section empty: {:?}", fetch.section(b"").map(|b| b.len()));
    }
    Ok(())
}
