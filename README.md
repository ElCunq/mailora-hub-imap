# Mailora Hub IMAP

A high-performance, real-time email synchronization service built with Rust. Mailora Hub provides unified email management across multiple providers with OAuth2 support, real-time notifications, and RESTful APIs.

## 🚀 Features

### ✅ Implemented
- **Multi-Provider Support**: Gmail, Yahoo, iCloud, and custom IMAP/SMTP
- **OAuth2 Authentication**: Secure passwordless login for Gmail (with PKCE)
- **Real-time Sync**: IMAP IDLE support for instant email notifications
- **Thread Management**: Automatic email threading and conversation grouping
- **Attachment Handling**: Download and manage email attachments
- **RESTful API**: Complete REST API for email operations
- **Event Streaming**: Server-Sent Events (SSE) for real-time updates
- **SQLite Database**: Efficient local storage with full-text search
- **Web UI**: Modern dark-themed interface for email management

### 🔧 Architecture
- **Backend**: Rust with Axum web framework
- **Database**: SQLite with async SQLx driver
- **Email**: async-imap for IMAP, lettre for SMTP
- **OAuth**: oauth2 crate with PKCE support
- **Logging**: Tracing with subscriber-based telemetry

## 📦 Installation

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- SQLite 3.35+

### Quick Start

1. **Clone the repository**
```bash
git clone https://github.com/YOUR_USERNAME/mailora-hub-imap.git
cd mailora-hub-imap
```

2. **Configure environment**
```bash
# Create .env file
cat > .env << EOF
DATABASE_URL=sqlite://mailora_imap.db

# Google OAuth2 (Gmail)
GOOGLE_CLIENT_ID=your_google_client_id
GOOGLE_CLIENT_SECRET=your_google_client_secret
EOF
```

3. **Build and run**
```bash
cargo build --release
./target/release/mailora-hub-imap
```

4. **Access the UI**
```
http://localhost:3030/static/app.html
```

## 🔐 OAuth2 Setup

### Gmail OAuth2

1. Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. Create OAuth 2.0 Client ID (Web application)
3. Add authorized redirect URI:
   ```
   http://localhost:3030/oauth/callback
   ```
4. Add authorized JavaScript origin:
   ```
   http://localhost:3030
   ```
5. Enable Gmail API in [API Library](https://console.cloud.google.com/apis/library/gmail.googleapis.com)
6. Copy Client ID and Client Secret to `.env`

See [memory_bank/OAUTH_SETUP.md](./memory_bank/OAUTH_SETUP.md) for detailed instructions.

## 🎯 Usage

### Adding an Account

**Option 1: OAuth2 (Recommended for Gmail)**
1. Open http://localhost:3030/static/app.html
2. Go to "➕ Hesap Ekle" tab
3. Select "Gmail" provider
4. Enter your email address
5. Click "🔐 OAuth2 ile Giriş"
6. Authorize in the popup window

**Option 2: App Password (Gmail, Yahoo, iCloud)**
1. Generate an app-specific password from your provider
2. Select provider and enter email
3. Paste the app password
4. Click "✅ Hesap Ekle"

### Syncing Emails
1. Go to "Hesaplar" tab
2. Click "📥 Sync" on your account
3. Watch real-time sync progress

### Real-time Notifications
1. Go to "IDLE" tab
2. Select account
3. Click "🟢 IDLE Başlat"
4. Receive instant notifications for new emails

## 📡 API Endpoints

### Accounts
- `GET /accounts` - List all accounts
- `POST /accounts` - Add new account (password or OAuth2)
- `GET /accounts/:id` - Get account details
- `DELETE /accounts/:id` - Remove account

### Messages
- `GET /messages/unified` - Get unified inbox
- `GET /messages/:account_id` - Get messages for account
- `GET /messages/:account_id/:message_id/body` - Get message body
- `POST /messages/:account_id/:message_id/action` - Mark as read/unread/delete

### OAuth
- `GET /oauth/start?provider=gmail` - Start OAuth flow
- `GET /oauth/callback` - OAuth callback handler

### Events
- `GET /events` - Server-Sent Events stream

## 🗂️ Database Schema

```sql
accounts (id, email, provider, imap_host, smtp_host, auth_method, oauth_tokens)
mailboxes (id, account_id, name, path, attributes)
messages (id, account_id, mailbox_id, uid, subject, from, date, flags)
threads (id, subject_hash, participants, message_count)
attachments (id, message_id, filename, content_type, size)
```

## 🛠️ Development

### Project Structure
```
src/
├── main.rs              # Entry point
├── config.rs            # Configuration
├── db/                  # Database queries
├── imap/                # IMAP sync logic
│   ├── conn.rs          # Connection handling
│   ├── sync.rs          # Message sync
│   ├── idle.rs          # Real-time IDLE
│   └── xoauth2.rs       # OAuth2 SASL
├── models/              # Data models
├── routes/              # API endpoints
│   ├── accounts.rs      # Account management
│   ├── oauth.rs         # OAuth2 flow
│   └── events.rs        # SSE streaming
└── services/            # Business logic

migrations/              # SQL migrations
static/                  # Web UI
memory_bank/            # Documentation & specs
```

### Running Tests
```bash
cargo test
cargo test --test integration_*
```

### Building for Production
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

## 🔒 Security

- **OAuth2 with PKCE**: Protects against authorization code interception
- **Encrypted Credentials**: Passwords stored with base64 encoding (upgrade to proper encryption recommended)
- **HTTPS Ready**: Configure with TLS certificates for production
- **Token Refresh**: Automatic OAuth2 token refresh before expiration

## 📚 Documentation

- [OAuth Setup Guide](./memory_bank/OAUTH_SETUP.md) - Detailed OAuth2 configuration
- [Quick Start](./memory_bank/QUICK_START.md) - Get started in 5 minutes
- [Architecture](./memory_bank/ARCHITECTURE.md) - System design overview
- [Test Results](./memory_bank/TEST_RESULTS.md) - Testing documentation

## 🐛 Known Issues

- [ ] Outlook OAuth2 not implemented (requires Azure AD setup)
- [ ] async-imap doesn't support XOAUTH2 natively (workaround in place)
- [ ] Credential encryption needs proper implementation (currently base64)
- [ ] Token refresh logic needs testing

## 🚧 Roadmap

- [ ] End-to-end encryption for stored credentials
- [ ] Outlook/Microsoft 365 OAuth2 support
- [ ] Email composition and sending
- [ ] Full-text search across all messages
- [ ] Email rules and filters
- [ ] Desktop notifications
- [ ] Mobile app (React Native)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License.

## 👥 Authors

- **Cenk Orfa** - Initial work

## 🙏 Acknowledgments

- [async-imap](https://github.com/async-email/async-imap) - Async IMAP client
- [lettre](https://github.com/lettre/lettre) - SMTP client
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit

---

**⚠️ Development Status**: This project is under active development. Use in production at your own risk.
