# Roadmap (Updated 2025-11-10)

Near-term (v0.2.x)
- Attachments pipeline
  - Parse BODYSTRUCTURE, persist to `attachments` table
  - `GET /attachments/:account/:folder/:uid` list
  - `GET /attachments/:account/:folder/:uid/:part` download
- Unified search API
  - Endpoint `/unified/search` with filters (q, from, to, date_from, date_to, unread, limit, offset)
  - DB indices and optional IMAP SEARCH fallback
- Persistent queues
  - `pending_outbox`, `pending_sent` with retry/backoff & resume on restart
- Error envelope & client handling
  - Standardize `{ok:false, code, message, hint}`
- Reliability
  - Scheduler jitter + exponential backoff
  - IDLE reconnect policy and health checks
- Performance
  - Indices for `messages` and `attachments`

Mid-term (v0.3)
- Threading
  - Thread grouping by Message-Id/References/In-Reply-To
  - `/threads` and thread detail endpoints
- Search indexing
  - SQLite FTS5 body index and previews
- Multi-device state
  - Last-seen markers, per-device cursors

Nice-to-have
- OAuth2 re-introduction for Gmail/Outlook
- Theming and keyboard shortcuts in UI
- Export/import mailbox (mbox/EML)
