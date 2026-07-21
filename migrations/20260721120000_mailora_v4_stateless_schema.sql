-- Drop messages and message body caching tables
DROP TABLE IF EXISTS messages_fts;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS message_bodies;

-- Drop obsolete synchronization state tracking tables
DROP TABLE IF EXISTS mailbox_sync_states;
DROP TABLE IF EXISTS sync_runs;
