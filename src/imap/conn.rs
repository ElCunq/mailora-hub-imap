use anyhow::Result;
use async_imap::Session;
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub struct ImapCapabilities {
    pub condstore: bool,
    pub qresync: bool,
}

pub struct ImapSession {
    pub session: Session<tokio_util::compat::Compat<tokio_native_tls::TlsStream<TcpStream>>>,
    pub caps: ImapCapabilities,
}

pub async fn connect(host: &str, port: u16, user: &str, pass: &str) -> Result<ImapSession> {
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let session = client
        .login(user, pass)
        .await
        .map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
    // Minimal capability flags (can implement proper parsing later)
    let caps = ImapCapabilities {
        condstore: false,
        qresync: false,
    };
    Ok(ImapSession { session, caps })
}
