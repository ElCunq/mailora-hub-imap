# Quick Start Guide

## Prerequisites
- Rust (latest stable)
- Docker (optional, but recommended)

## Zero to Running
1. **Clone & Run:**
   ```bash
   git clone <repo_url>
   cd mailora-hub-imap
   cargo run
   ```
   Server listens on `http://localhost:3030`.

2. **Access Web UI:**
   Open [http://localhost:3030/app.html](http://localhost:3030/app.html).

3. **Add First Account:**
   - Click **+ Yeni Hesap Ekle**.
   - **Magic Login:** Enter email/password (e.g., Gmail with App Password).
   - Click "Giriş Yap". Settings are auto-discovered!

4. **Verify Sync:**
   - Emails appear in the inbox.
   - Click on an email to view body & attachments.
   - Send a test email via "Compose" button.

## Troubleshooting
- **Port Conflict:** If 3030 is busy, `cargo run` will kill the old process automatically.
- **Auto-Discovery Fails:** Click "Manuel Kurulum" and enter IMAP/SMTP details manually.
- **Gmail:** Ensure 2FA is ON and use an **App Password**.
