# Milestone 2 Status (2025-11-10)

Scope delivered (v0.2.1)
- Unified Inbox (folder-based aggregation across accounts) with unread filter and pagination (backend route `/unified/inbox`).
- Background scheduler with per-account frequency and safe skips (Gmail, empty/invalid creds).
- Initial full sync on startup (non-blocking) for enabled, non-Gmail accounts.
- Message body cache with TTL+cap GC and force refresh.
- Flags update route with insert fallback.
- Minimal 3-pane Web UI (accounts/folders, message list, preview+compose).
- SMTP send + APPEND policy (auto/never/force) behavior wired.

Deferrals
- Gmail-specific logic fully tested end-to-end.
- Persistent sent-finalize queue across restarts.
- Attachments fetch + download.
- Full-text search + advanced unified search filters.
- Consistent error envelope across all endpoints.
- Metrics correctness.

Known constraints
- DB schema is accessed via dynamic SQL to tolerate drift; ensure proper indices before load tests.
- OAuth/JMAP flows are sidelined for now.

Next targets
1) Attachments pipeline: parse, persist, list, download endpoints.
2) Unified search API: subject/from/to/date filters + pagination.
3) Persistent finalize queues (outbox, sent-finalize) with retry/backoff.
4) Error envelope standardization and client handling.
5) Scheduler backoff/jitter + IDLE reconnect strategy.
6) Indexing: messages(account_id,folder,uid), messages(folder,date DESC), attachments(message_id).
