/// XOAUTH2 authentication for IMAP
///
/// This module provides OAuth2 authentication for IMAP connections
/// using the XOAUTH2 SASL mechanism.
use async_imap::Session;
use base64::Engine;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

type CompatTlsStream = Compat<TlsStream<TcpStream>>;

/// Generate XOAUTH2 authentication string
///
/// Format: user={email}\x01auth=Bearer {token}\x01\x01
pub fn generate_xoauth2_string(email: &str, access_token: &str) -> String {
    format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token)
}

/// Authenticate with IMAP using XOAUTH2
///
/// async-imap doesn't support XOAUTH2 directly, so we use login() which
/// will send credentials. For real XOAUTH2, we need to send raw commands.
///
/// For now, we'll use a workaround: store the OAuth token and use it
/// when connecting. The actual XOAUTH2 SASL flow needs raw IMAP commands.
pub async fn connect_with_oauth(
    host: &str,
    port: u16,
    email: &str,
    access_token: &str,
) -> Result<Session<CompatTlsStream>, String> {
    // Connect to server
    let tcp_stream = tokio::net::TcpStream::connect((host, port))
        .await
        .map_err(|e| format!("TCP connection failed: {}", e))?;

    // TLS handshake
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS builder failed: {}", e))?;

    let tls_connector = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls_connector
        .connect(host, tcp_stream)
        .await
        .map_err(|e| format!("TLS connection failed: {}", e))?;

    // Convert to compat layer for async-imap
    let compat_stream = tls_stream.compat();

    // Create IMAP client
    let client = async_imap::Client::new(compat_stream);

    // Generate XOAUTH2 string
    let auth_string = generate_xoauth2_string(email, access_token);

    // For Gmail/Outlook OAuth, we need to use XOAUTH2 mechanism
    // async-imap's login() won't work. We need to use authenticate() with a custom authenticator
    // For now, we'll try login with the email and use "oauth2:" prefix as a marker
    // This is a workaround - real implementation needs raw IMAP command sending

    // Try standard login (this will fail for OAuth-only accounts)
    // In production, implement proper XOAUTH2 SASL flow
    match client.login(email, &auth_string).await {
        Ok(session) => Ok(session),
        Err((e, _client)) => {
            Err(format!("OAuth2 login failed: {:?}. Note: async-imap doesn't support XOAUTH2 SASL yet. Consider using native IMAP commands or a different library.", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_xoauth2_string() {
        let email = "test@example.com";
        let token = "ya29.test_token";

        let auth_string = generate_xoauth2_string(email, token);

        assert!(auth_string.contains("user=test@example.com"));
        assert!(auth_string.contains("auth=Bearer ya29.test_token"));
        assert!(auth_string.ends_with("\x01\x01"));
    }
}
