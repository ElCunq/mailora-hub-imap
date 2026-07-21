-- Phase 6 DB Optimizations: Extract flag columns, folder_kind, internal_date_ts, and create performance indexes

-- 1. Extract flag columns and folder_kind on messages table
ALTER TABLE messages ADD COLUMN is_seen BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE messages ADD COLUMN is_answered BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE messages ADD COLUMN is_flagged BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE messages ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE messages ADD COLUMN is_draft BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE messages ADD COLUMN folder_kind TEXT;
ALTER TABLE messages ADD COLUMN internal_date_ts INTEGER DEFAULT 0;

-- 2. Backfill existing flags from JSON string and set folder_kind
UPDATE messages SET is_seen = 1 WHERE flags LIKE '%\\Seen%';
UPDATE messages SET is_answered = 1 WHERE flags LIKE '%\\Answered%';
UPDATE messages SET is_flagged = 1 WHERE flags LIKE '%\\Flagged%';
UPDATE messages SET is_deleted = 1 WHERE flags LIKE '%\\Deleted%';
UPDATE messages SET is_draft = 1 WHERE flags LIKE '%\\Draft%';

UPDATE messages SET folder_kind = 'inbox' WHERE lower(folder) = 'inbox';
UPDATE messages SET folder_kind = 'sent' WHERE lower(folder) = 'sent' OR lower(folder) LIKE '%sent mail%' OR lower(folder) LIKE '%sent items%';
UPDATE messages SET folder_kind = 'drafts' WHERE lower(folder) = 'drafts';
UPDATE messages SET folder_kind = 'trash' WHERE lower(folder) = 'trash' OR lower(folder) LIKE '%deleted%' OR lower(folder) = 'bin';
UPDATE messages SET folder_kind = 'junk' WHERE lower(folder) = 'junk' OR lower(folder) = 'spam';
UPDATE messages SET folder_kind = 'archive' WHERE lower(folder) = 'archive' OR lower(folder) = 'archives';

-- 3. Backfill internal_date_ts from unixepoch if internal_date or date string is available
UPDATE messages SET internal_date_ts = strftime('%s', date) WHERE internal_date_ts = 0 AND date IS NOT NULL AND date != '';

-- 4. Performance indexes for unified inbox and cursor/keyset pagination
CREATE INDEX IF NOT EXISTS idx_messages_unified_inbox ON messages(account_id, folder_kind, internal_date_ts DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_messages_flags_boolean ON messages(account_id, is_seen, is_flagged);
CREATE INDEX IF NOT EXISTS idx_messages_cursor_paging ON messages(internal_date_ts DESC, id DESC);

-- 5. Fix FTS5 external content triggers to prevent database corruption on UPDATE/DELETE
DROP TRIGGER IF EXISTS messages_ad;
DROP TRIGGER IF EXISTS messages_au;

CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, subject, from_addr, to_addr, body_plain)
    VALUES('delete', old.id, old.subject, old.from_addr, old.to_addr, old.body_plain);
END;

CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, subject, from_addr, to_addr, body_plain)
    VALUES('delete', old.id, old.subject, old.from_addr, old.to_addr, old.body_plain);
    INSERT INTO messages_fts(rowid, subject, from_addr, to_addr, body_plain)
    VALUES (new.id, new.subject, new.from_addr, new.to_addr, new.body_plain);
END;
