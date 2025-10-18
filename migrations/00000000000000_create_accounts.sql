-- Core accounts table required by seed_account
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    org_id INTEGER NOT NULL,
    email TEXT NOT NULL UNIQUE,
    imap_host TEXT NOT NULL,
    imap_port INTEGER NOT NULL,
    smtp_host TEXT NOT NULL,
    smtp_port INTEGER NOT NULL,
    auth_type TEXT NOT NULL,
    use_ssl INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Minimal folders table referencing accounts for future use
CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    role TEXT,
    uidvalidity INTEGER,
    highest_modseq INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    UNIQUE(account_id, name),
    FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
);
