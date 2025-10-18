# Sonraki Adımlar - Milestone 1.3

## 🎯 Hedef: Message Persistence & SMTP Send

### Öncelik 1: Message Sync to SQLite (Database Persistence)

#### 1.1 Messages Table Schema
```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,              -- msg_{account_id}_{uid}
    account_id TEXT NOT NULL,
    uid INTEGER NOT NULL,
    folder TEXT NOT NULL,
    subject TEXT,
    from_addr TEXT,
    to_addr TEXT,
    cc_addr TEXT,
    date TEXT,
    body_html TEXT,
    body_plain TEXT,
    flags TEXT,                       -- JSON array: ["\\Seen", "\\Flagged"]
    thread_id TEXT,
    has_attachments INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    UNIQUE(account_id, uid, folder)
);

CREATE INDEX idx_messages_account ON messages(account_id);
CREATE INDEX idx_messages_folder ON messages(account_id, folder);
CREATE INDEX idx_messages_date ON messages(date DESC);
CREATE INDEX idx_messages_thread ON messages(thread_id);
```

#### 1.2 Attachments Table Schema
```sql
CREATE TABLE attachments (
    id TEXT PRIMARY KEY,              -- att_{message_id}_{index}
    message_id TEXT NOT NULL,
    filename TEXT,
    content_type TEXT,
    size INTEGER,
    storage_path TEXT,                -- Filesystem path: ./attachments/{id}
    created_at TEXT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_attachments_message ON attachments(message_id);
```

#### 1.3 Migration SQL
**File**: `migrations/002_messages_and_attachments.sql`

#### 1.4 Message Sync Service
**File**: `src/services/message_sync_service.rs`
```rust
pub async fn sync_messages_for_account(
    pool: &SqlitePool,
    account_id: &str,
    folder: &str,
    limit: usize
) -> Result<usize> {
    // 1. Fetch messages from IMAP
    // 2. Parse headers and body
    // 3. Upsert into messages table
    // 4. Extract and save attachments
    // 5. Return count of synced messages
}
```

#### 1.5 API Endpoints
- `POST /sync/account/:id` - Trigger full sync for account
- `POST /sync/account/:id/folder/:folder` - Sync specific folder
- `GET /sync/status/:id` - Get sync progress
- `GET /messages/search?q=term` - Search in synced messages

---

### Öncelik 2: SMTP Send Service

#### 2.1 Add Lettre Dependency
```toml
lettre = { version = "0.11", features = ["tokio1", "tokio1-native-tls", "smtp-transport"] }
```

#### 2.2 SMTP Service
**File**: `src/services/smtp_service.rs`
```rust
pub async fn send_email(
    account: &Account,
    to: Vec<String>,
    subject: String,
    body_html: String,
    body_plain: Option<String>,
    attachments: Vec<Attachment>
) -> Result<()> {
    // 1. Parse provider SMTP config
    // 2. Build email with lettre
    // 3. Send via SMTP
    // 4. Copy to Sent folder via IMAP
}
```

#### 2.3 API Endpoints
- `POST /send` - Send email
  ```json
  {
    "account_id": "acc_xxx",
    "to": ["recipient@example.com"],
    "cc": [],
    "bcc": [],
    "subject": "Hello",
    "body_html": "<p>Hi there</p>",
    "body_plain": "Hi there",
    "attachments": []
  }
  ```

#### 2.4 UI Updates
- Add "Compose" button in app.html
- Compose modal with rich text editor (TODO: use Quill.js or similar)
- Send progress indicator
- Sent confirmation

---

### Öncelik 3: Attachment Handling

#### 3.1 Attachment Extraction
**In `message_sync_service.rs`:**
```rust
async fn extract_attachments(
    message: &Message,
    message_id: &str
) -> Result<Vec<AttachmentInfo>> {
    // Parse MIME parts
    // Identify attachments (Content-Disposition: attachment)
    // Save to ./attachments/{id}
    // Return metadata for DB insert
}
```

#### 3.2 Attachment Storage
- Filesystem: `./attachments/{attachment_id}`
- OR: Store in SQLite as BLOB (for small files < 1MB)
- Implement cleanup for deleted messages

#### 3.3 API Endpoints
- `GET /attachments/:id/download` - Download attachment
- `GET /attachments/:id/preview` - Preview (images only)

---

### Öncelik 4: Thread Grouping

#### 4.1 Thread Detection
```rust
fn generate_thread_id(
    subject: &str,
    references: &[String],
    in_reply_to: &Option<String>
) -> String {
    // Use In-Reply-To and References headers
    // OR: Subject-based grouping (Gmail-style)
    // Return consistent thread_id
}
```

#### 4.2 API Enhancement
- `GET /threads/:account_id` - List threads
- `GET /threads/:thread_id/messages` - Messages in thread

---

### Öncelik 5: Full-Text Search

#### 5.1 SQLite FTS5 Table
```sql
CREATE VIRTUAL TABLE messages_fts USING fts5(
    subject,
    from_addr,
    to_addr,
    body_plain,
    content=messages,
    content_rowid=rowid
);

-- Triggers to keep FTS in sync
CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, subject, from_addr, to_addr, body_plain)
    VALUES (new.rowid, new.subject, new.from_addr, new.to_addr, new.body_plain);
END;
```

#### 5.2 Search API
- `GET /search?q=urgent+project&account=acc_xxx` - Full-text search

---

### Öncelik 6: Unified Inbox

#### 6.1 Aggregation Query
```sql
SELECT * FROM messages
WHERE account_id IN (SELECT id FROM accounts WHERE enabled=1)
ORDER BY date DESC
LIMIT 50;
```

#### 6.2 UI Enhancement
- Add "Unified Inbox" tab
- Show messages from all accounts
- Color-code by account
- Filter by account/folder

---

## 📅 Timeline Tahmini

| Milestone | Tahmini Süre | Öncelik |
|-----------|--------------|---------|
| Message Sync to DB | 2-3 gün | ⭐⭐⭐ Kritik |
| SMTP Send Service | 1-2 gün | ⭐⭐⭐ Kritik |
| Attachment Handling | 1-2 gün | ⭐⭐ Yüksek |
| Thread Grouping | 1 gün | ⭐ Orta |
| Full-Text Search | 1 gün | ⭐ Orta |
| Unified Inbox UI | 1 gün | ⭐⭐ Yüksek |

**Toplam**: 7-10 gün (part-time çalışma varsayımıyla)

---

## 🚀 Hemen Başlanabilecek

### Adım 1: Migration Oluştur
```bash
mkdir -p migrations
touch migrations/002_messages_and_attachments.sql
```

### Adım 2: Message Sync Service
```bash
touch src/services/message_sync_service.rs
```

### Adım 3: Test Migration
```bash
sqlx migrate run
```

---

## ❓ Kararlar Gerekiyor

1. **Attachment Storage:**
   - [ ] Filesystem (önerilen)
   - [ ] SQLite BLOB
   - [ ] Hybrid (küçükler DB, büyükler FS)

2. **Thread Grouping:**
   - [ ] Header-based (Gmail-style, kesin)
   - [ ] Subject-based (basit ama hatalı olabilir)

3. **Search:**
   - [ ] SQLite FTS5 (built-in, hızlı)
   - [ ] External (Meilisearch, elasticsearch - overkill?)

4. **Rich Text Editor:**
   - [ ] Quill.js
   - [ ] TinyMCE
   - [ ] Vanilla contentEditable

---

**Hangi özellikle başlamak istersiniz?**
