# System Patterns & Architecture

## Core Architecture Principles

### 1. Database & Search Strategy
- **SQLite FTS5**: Full-Text Search is mandatory. We index `subject`, `from`, and `body_text` in a virtual table for millisecond-level search queries. `LIKE %query%` is prohibited for content search.
- **WAL Mode**: Write-Ahead Logging is enabled to allow concurrent readers and writers, preventing UI blocking during sync.
- **Maintenance**: Automated VACUUM/ANALYZE jobs run weekly to prevent fragmentation.

### 2. Protocol Optimization
- **IMAP IDLE**: We prefer "Push" over "Poll". Use RFC 2177 to keep TCP connections open and receive real-time updates.
- **Connection Pooling**: Re-use IMAP/SMTP connections via a pool (e.g., `deadpool`) to avoid expensive TLS handshakes for every action.

### 3. Resilience & Job Queue
- **Outbox Pattern**: Sending mail is async.
    1. UI saves mail to local `outbox` table.
    2. Background worker picks it up and attempts SMTP delivery.
    3. On failure -> Exponential Backoff (retry later).
    4. On success -> Move to "Sent" folder and delete from `outbox`.
- **Async Attachment Processing**: Heavy parsing of attachments is offloaded to `tokio::spawn` tasks to keep the critical path clear.

### 4. Zero-Copy & Memory
- Prefer zero-copy deserialization where possible to minimize RAM footprint during large fetches.

## Security Model
- **RBAC**: Strict separation of Admin vs Member.
- **Unified View**: MUST respect `user_accounts` visibility. A user sees aggregate views ONLY for accounts they are assigned to.
