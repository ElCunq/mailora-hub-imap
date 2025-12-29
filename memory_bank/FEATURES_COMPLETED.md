# Features Completed

## v1.1.0 - Auto-Discovery & UI Polishing (2025-12-25)
- **Auto-Discovery (Magic Login):**
  - Just email/password needed for most providers.
  - Queries Mozilla ISPDB.
  - DNS SRV record lookup (`_imap._tcp`, `_submission._tcp`).
  - Heuristic domain guessing.
  - Fallback to manual setup if all fails.
- **UI Unification:**
  - `add_account.html` now uses the application's global Dark Theme (`#0f1217`, `#1f2937`).
  - Centralized navigation: Sidebar button + dedicated page logic.
- **Stability:**
  - Circuit Breaker for Sync loops (stops infinite log spam).
  - Robust port cleanup on startup.

## v1.0.0 - Docker & Attachments (2025-12-16)
- **Docker Support:**
  - production-ready `Dockerfile` (multi-stage).
  - `docker-compose.yml` with persistent storage.
- **Attachment Management:**
  - Full metadata extraction.
  - On-demand download endpoints.
  - Inline image (CID) support for HTML emails.

## v0.3.0 - Multi-Account & Sync (2025-12-05)
- **Multi-Account Support:** Add/Delete/Edit accounts.
- **IMAP Sync:**
  - Delta sync using `UIDVALIDITY` and `LAST_UID`.
  - IDLE support for real-time notifications.
- **Database:**
  - SQLite schema for Accounts and Messages.
  - WAL mode enabled for concurrency.
