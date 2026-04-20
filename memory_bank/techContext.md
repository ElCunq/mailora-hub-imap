# Technical Context

## Stack
- **Backend**: Rust with Axum web framework
- **IMAP Client**: async-imap crate for direct connections
- **SMTP Client**: lettre crate
- **Database**: SQLite (accounts, messages, cache, attachments)
- **Async Runtime**: Tokio for concurrent connections
- **Frontend**: Vanilla HTML/CSS/JavaScript (no framework)
- **API**: REST (no JMAP/Stalwart dependency)
- **Containerization**: Docker, Docker Compose

## Key Dependencies
```toml
async-imap = "0.10"           # IMAP client
lettre = "0.11"               # SMTP client
sqlx = "0.8"                  # SQLite async driver
tokio = "1"                   # Async runtime
axum = "0.7"                  # Web framework
tower-http = "0.5"            # Static file serving
serde = "1.0"                 # JSON serialization
base64 = "0.22"               # Credential encoding
mailparse = "0.14"            # MIME parsing
ammonia = "4"                 # HTML sanitization
trust-dns-resolver = "0.23"   # DNS SRV lookups (discovery)
quick-xml = "0.36"            # ISPDB XML parsing
reqwest = "0.12"              # HTTP client (ISPDB)
```

## Database Schema

### Accounts Table
```sql
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    provider TEXT NOT NULL,
    credentials TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    sync_frequency_secs INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Messages Table (TODO)
```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    uid INTEGER NOT NULL,
    folder TEXT NOT NULL,
    subject TEXT,
    from_addr TEXT,
    to_addr TEXT,
    date TEXT,
    body_html TEXT,
    body_plain TEXT,
    flags TEXT,
    thread_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);
```

## Service Architecture

```
src/
├── services/
│   ├── account_service.rs      # Account CRUD operations
│   ├── imap_service.rs          # Base IMAP connection
│   ├── message_body_service.rs  # Message fetch + MIME parsing
│   ├── idle_watcher_service.rs  # Real-time IDLE watchers
│   └── imap_test_service.rs     # Testing endpoints
├── routes/
│   ├── accounts.rs              # Account API routes
│   ├── messages.rs              # Message API routes
│   └── idle.rs                  # IDLE watcher routes
├── models/
│   └── account.rs               # Account, Provider models
└── main.rs                      # Server entry point
```

## API Endpoints

### Accounts
- `POST /accounts` - Add account
- `GET /accounts` - List accounts
- `GET /accounts/:id` - Get account
- `DELETE /accounts/:id` - Delete account
- `GET /providers` - Provider presets

### Messages
- `GET /test/messages/:account_id` - Fetch messages
- `GET /messages/:account_id/:uid/body` - Get message body

### IDLE
- `POST /idle/:account_id/start` - Start IDLE watcher
- `POST /idle/:account_id/stop` - Stop watcher
- `GET /idle/:account_id/status` - Check status

## Security Considerations

- **Credentials**: Base64 encoded (temporary) → Move to OS keychain
- **IMAP/SMTP**: TLS connections enforced
- **Local Storage**: SQLite file permissions
- **No Cloud**: All data stays local

## Performance

- **Async I/O**: Tokio for non-blocking operations
- **Connection Pooling**: SQLite connection pool (SQLx)
- **IDLE Efficiency**: One connection per account, minimal bandwidth
- **Cache**: SQLite for fast message retrieval
