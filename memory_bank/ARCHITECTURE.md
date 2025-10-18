# Architecture

## System Overview

Mailora Hub IMAP is a unified email client that aggregates multiple external email accounts (Gmail, Outlook, Yahoo, etc.) into a single interface using direct IMAP/SMTP connections and SQLite for local storage.

## Architecture Layers

```
┌─────────────────────────────────────────────────┐
│           Frontend (Web UI)                     │
│              REST API Client                    │
└────────────────┬────────────────────────────────┘
                 │
                 │ HTTP/REST
                 │
┌────────────────▼────────────────────────────────┐
│         Mailora Backend (Rust/Axum)            │
│  - REST API Endpoints                          │
│  - Account Management                          │
│  - Message Caching & Search                    │
│  - SQLite Database                             │
└────────────────┬────────────────────────────────┘
                 │
                 │ async-imap / lettre
                 │
┌────────────────▼────────────────────────────────┐
│         IMAP/SMTP Services (Rust)              │
│  - Multi-account IMAP client (async-imap)      │
│  - IDLE watchers (real-time notifications)     │
│  - Message body fetch & MIME parsing           │
│  - SMTP send service (lettre) [TODO]          │
└────────────────┬────────────────────────────────┘
                 │
                 │ IMAP/SMTP Protocols
                 │
┌────────────────▼────────────────────────────────┐
│      External Email Providers                  │
│  Gmail, Outlook, Yahoo, iCloud, Custom         │
└─────────────────────────────────────────────────┘
```

## Core Components

### 1. Mailora Backend (Rust/Axum)
- **Role**: Central REST API server and business logic
- **Functions**:
  - Account management (CRUD operations)
  - Direct IMAP connections to external providers
  - Message caching in SQLite
  - Real-time IDLE watchers per account
  - SMTP send service (TODO)
  - Full-text search (SQLite FTS5) (TODO)

### 2. IMAP/SMTP Services (Rust)
- **Role**: Direct protocol implementation for email providers
- **Functions**:
  - Multi-account IMAP client (`async-imap` crate)
  - IMAP IDLE for real-time notifications
  - Message body fetch with MIME parsing
  - SMTP sending (`lettre` crate) (TODO)
  - Connection pooling and retry logic

### 3. SQLite Database
- **Role**: Local storage and unified index
- **Functions**:
  - Account credentials (base64 encoded)
  - Message metadata and content cache
  - Thread grouping (TODO)
  - Attachment storage (TODO)
  - Full-text search index (TODO)

### 4. Web UI (HTML/CSS/JS)
- **Role**: User interface
- **Functions**:
  - Account list from database
  - Unified inbox from all accounts
  - Message reading (HTML/Plain text)
  - IDLE watcher controls
  - Compose & send (TODO)

## Data Flow

### Email Fetch Flow
```
1. External IMAP (Gmail) → async-imap client
2. Message headers/body fetched → MIME parsing
3. Store in SQLite → Cache for quick access
4. Web UI queries REST API → Display to user
```

### Real-time Notifications
```
1. IDLE watcher per account → Background tokio task
2. New message notification → Event to frontend (TODO: WebSocket)
3. Fetch new message → Update SQLite cache
4. UI auto-refresh → Show new message
```

### Sending Email Flow (TODO)
```
1. User composes in Web UI → POST /send
2. Backend uses `lettre` → Original account's SMTP
3. Sent message → Sync back from IMAP Sent folder
4. Update SQLite → UI shows in Sent
```

## Technology Stack

- **Rust**: Backend language
- **Axum**: Web framework for REST API
- **async-imap**: IMAP client library for email fetching
- **lettre**: SMTP client library for sending (TODO)
- **SQLite**: Primary database (accounts, messages, cache)
- **Tokio**: Async runtime for concurrent IMAP connections
- **Tower-HTTP**: Static file serving for Web UI
- **HTML/CSS/JS**: Frontend (no framework, vanilla)

## Design Principles

1. **Simplicity**: Direct IMAP/SMTP, no intermediate mail server
2. **Performance**: SQLite cache for fast queries, async I/O
3. **Privacy**: Local storage, no cloud dependency
4. **Extensibility**: Modular service architecture
5. **Reliability**: Per-account error isolation, auto-reconnect
