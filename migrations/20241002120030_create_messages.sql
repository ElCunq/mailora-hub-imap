-- Unified message index across all accounts
CREATE TABLE messages (
    account_id TEXT NOT NULL,
    folder TEXT NOT NULL,
    uid INTEGER NOT NULL,
    msg_id TEXT,
    thread_key TEXT,
    subject TEXT,
    from_addr TEXT,
    to_addrs TEXT,
    internaldate INTEGER,
    flags TEXT,
    size INTEGER,
    PRIMARY KEY (account_id, folder, uid)
);

CREATE INDEX IF NOT EXISTS idx_unified ON messages(folder, internaldate DESC);
CREATE INDEX IF NOT EXISTS idx_msg_id ON messages(msg_id);
CREATE INDEX IF NOT EXISTS idx_thread ON messages(thread_key);