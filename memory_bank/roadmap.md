# Roadmap (Updated 2025-12-25)

## Near-term (v1.1.x)
- [COMPLETE] Auto-Discovery (Magic Login)
- [COMPLETE] UI Unification (Dark Theme)
- [COMPLETE] Attachments pipeline
- **Discovery Service Improvements:** Better heuristic fallback for obscure domains.
- **PIM Foundation:** Database schema for Contacts and Calendars.

## Mid-term (v1.2.0)
- **Full-Text Search (FTS):**
  - SQLite FTS5 integration.
  - Indexing logic for subjects and bodies.
- **Unified Search API:**
  - Filtering across all accounts.
- **Persistent Queues:**
  - `outbox` table for reliable sending.

## Long-term (v2.0)
- **Multi-Device Sync:** State shared across desktop/mobile.
- **Plugin System:** Allow third-party extensions.
- **Theming:** User-customizable CSS.
- **Stalwart Integration (Personal Cloud):** 
  - Transition local-only users to an embedded Stalwart server.
  - Use Stalwart JMAP/REST API for performance boost.
  - Existing "Universal IMAP" mode will remain for external accounts (Gmail/Outlook).

## Nice-to-have
- Export/import mailbox (mbox/EML).
- Keyboard shortcuts in UI.
- OAuth2 re-introduction for Gmail/Outlook (if App Passwords become obsolete).
