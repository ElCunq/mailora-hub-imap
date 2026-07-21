use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use anyhow::Result;
use base64::Engine;
use rand::RngCore;

fn key_from_env() -> Option<[u8; 32]> {
    if let Ok(key_b64) = std::env::var("MAILORA_KEY") {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(key_b64) {
            if bytes.len() == 32 {
                let mut k = [0u8; 32];
                k.copy_from_slice(&bytes);
                return Some(k);
            }
        }
    }
    None
}

pub fn encrypt_secret(plain: &str) -> String {
    if let Some(key) = key_from_env() {
        let cipher = Aes256Gcm::new((&key).into());
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        if let Ok(ct) = cipher.encrypt(nonce, plain.as_bytes()) {
            let mut blob = Vec::with_capacity(1 + 12 + ct.len());
            blob.push(1u8); // v1 prefix
            blob.extend_from_slice(&nonce_bytes);
            blob.extend_from_slice(&ct);
            return base64::engine::general_purpose::STANDARD.encode(&blob);
        }
    }
    base64::engine::general_purpose::STANDARD.encode(plain.as_bytes())
}

pub fn decrypt_secret(s: &str) -> Result<String> {
    let bytes = match base64::engine::general_purpose::STANDARD.decode(s) {
        Ok(b) => b,
        Err(_) => return Ok(s.to_string()), // If not valid base64, return raw string
    };
    if bytes.first() == Some(&1u8) && bytes.len() > 13 {
        if let Some(key) = key_from_env() {
            let (_, rest) = bytes.split_first().unwrap();
            let (nonce_bytes, ct) = rest.split_at(12);
            let cipher = Aes256Gcm::new((&key).into());
            let nonce = Nonce::from_slice(nonce_bytes);
            if let Ok(pt) = cipher.decrypt(nonce, ct) {
                return Ok(String::from_utf8(pt)?);
            }
        }
    }
    // Fallback if base64 encoded plain or unencrypted
    if let Ok(s_utf8) = String::from_utf8(bytes) {
        Ok(s_utf8)
    } else {
        Ok(s.to_string())
    }
}
