-- Mailora v3 Core Database Schema

-- 0. Users Table Reconciliation & v3 Schema Enforcement
ALTER TABLE users ADD COLUMN email TEXT;
ALTER TABLE users ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
UPDATE users SET email = username WHERE email IS NULL AND username IS NOT NULL;
UPDATE users SET role = 'SuperAdmin' WHERE role IN ('Admin', 'SuperAdmin');
UPDATE users SET role = 'User' WHERE role IN ('Member', 'User') OR role IS NULL;

-- 1. Mailcow Instances
CREATE TABLE IF NOT EXISTS mailcow_instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    imap_host TEXT NOT NULL,
    imap_port INTEGER NOT NULL DEFAULT 993,
    smtp_host TEXT NOT NULL,
    smtp_port INTEGER NOT NULL DEFAULT 587,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    last_discovery_at TEXT,
    last_discovery_status TEXT,
    last_discovery_error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. Domains (Discovered from Mailcow)
CREATE TABLE IF NOT EXISTS domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mailcow_instance_id INTEGER NOT NULL,
    external_id TEXT,
    name TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    quota INTEGER,
    mailbox_limit INTEGER,
    last_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(mailcow_instance_id, name),
    FOREIGN KEY(mailcow_instance_id) REFERENCES mailcow_instances(id)
);

-- 3. Mailboxes (Discovered from Mailcow)
CREATE TABLE IF NOT EXISTS mailboxes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    external_id TEXT,
    address TEXT NOT NULL UNIQUE,
    local_part TEXT NOT NULL,
    display_name TEXT,
    active BOOLEAN NOT NULL DEFAULT 1,
    quota INTEGER,
    used_quota INTEGER,
    credential_status TEXT NOT NULL DEFAULT 'missing',
    connection_status TEXT NOT NULL DEFAULT 'not_tested',
    sync_status TEXT NOT NULL DEFAULT 'idle',
    last_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY(domain_id) REFERENCES domains(id)
);

-- 3.1. Mailboxes Table Reconciliation for legacy upgrades
ALTER TABLE mailboxes ADD COLUMN domain_id INTEGER DEFAULT 0;
ALTER TABLE mailboxes ADD COLUMN external_id TEXT;
ALTER TABLE mailboxes ADD COLUMN local_part TEXT DEFAULT '';
ALTER TABLE mailboxes ADD COLUMN display_name TEXT;
ALTER TABLE mailboxes ADD COLUMN active BOOLEAN DEFAULT 1;
ALTER TABLE mailboxes ADD COLUMN quota INTEGER;
ALTER TABLE mailboxes ADD COLUMN used_quota INTEGER;
ALTER TABLE mailboxes ADD COLUMN credential_status TEXT DEFAULT 'missing';
ALTER TABLE mailboxes ADD COLUMN connection_status TEXT DEFAULT 'not_tested';
ALTER TABLE mailboxes ADD COLUMN sync_status TEXT DEFAULT 'idle';
ALTER TABLE mailboxes ADD COLUMN last_seen_at TEXT DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE mailboxes ADD COLUMN deleted_at TEXT;
UPDATE mailboxes SET local_part = address WHERE local_part = '' OR local_part IS NULL;

-- 4. Domain Admin Assignments
CREATE TABLE IF NOT EXISTS domain_admin_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    domain_id INTEGER NOT NULL,
    assigned_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    revoked_at TEXT,
    UNIQUE(user_id, domain_id),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(domain_id) REFERENCES domains(id)
);

-- 5. Mailbox Assignments & Granular Permissions
CREATE TABLE IF NOT EXISTS mailbox_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    mailbox_id INTEGER NOT NULL,
    can_view BOOLEAN NOT NULL DEFAULT 0,
    can_read BOOLEAN NOT NULL DEFAULT 0,
    can_reply BOOLEAN NOT NULL DEFAULT 0,
    can_send BOOLEAN NOT NULL DEFAULT 0,
    can_send_as BOOLEAN NOT NULL DEFAULT 0,
    can_mark_read BOOLEAN NOT NULL DEFAULT 0,
    can_move BOOLEAN NOT NULL DEFAULT 0,
    can_delete BOOLEAN NOT NULL DEFAULT 0,
    can_manage BOOLEAN NOT NULL DEFAULT 0,
    assigned_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    revoked_at TEXT,
    UNIQUE(user_id, mailbox_id),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(mailbox_id) REFERENCES mailboxes(id)
);

-- 6. Mailbox Credentials (AES-GCM Encrypted)
CREATE TABLE IF NOT EXISTS mailbox_credentials (
    mailbox_id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    password_encrypted TEXT NOT NULL,
    credential_type TEXT NOT NULL DEFAULT 'password',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_verified_at TEXT,
    verification_status TEXT,
    verification_error TEXT,
    FOREIGN KEY(mailbox_id) REFERENCES mailboxes(id)
);

-- 7. Mailbox Sync States (Incremental Sync Tracking)
CREATE TABLE IF NOT EXISTS mailbox_sync_states (
    mailbox_id INTEGER NOT NULL,
    folder_name TEXT NOT NULL,
    folder_kind TEXT,
    uid_validity INTEGER NOT NULL DEFAULT 0,
    highest_uid INTEGER NOT NULL DEFAULT 0,
    highest_modseq INTEGER,
    last_incremental_sync_at TEXT,
    last_reconciliation_at TEXT,
    last_success_at TEXT,
    last_error TEXT,
    sync_status TEXT NOT NULL DEFAULT 'idle',
    PRIMARY KEY(mailbox_id, folder_name),
    FOREIGN KEY(mailbox_id) REFERENCES mailboxes(id)
);

-- 8. Job Locks (Concurrency Control)
CREATE TABLE IF NOT EXISTS job_locks (
    lock_key TEXT PRIMARY KEY,
    locked_by TEXT NOT NULL,
    locked_until TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 9. Audit Logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    actor_user_id INTEGER,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    domain_id INTEGER,
    mailbox_id INTEGER,
    metadata_json TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 10. Sync Runs Log
CREATE TABLE IF NOT EXISTS sync_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mailbox_id INTEGER NOT NULL,
    folder_name TEXT NOT NULL,
    sync_type TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at TEXT,
    duration_ms INTEGER,
    new_count INTEGER DEFAULT 0,
    updated_count INTEGER DEFAULT 0,
    deleted_count INTEGER DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    error_message TEXT,
    FOREIGN KEY(mailbox_id) REFERENCES mailboxes(id)
);

-- 11. Send Audit Log
CREATE TABLE IF NOT EXISTS send_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    actor_user_id INTEGER NOT NULL,
    mailbox_id INTEGER NOT NULL,
    from_address TEXT NOT NULL,
    recipients TEXT NOT NULL,
    message_id TEXT,
    smtp_response TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    sent_at TEXT,
    FOREIGN KEY(actor_user_id) REFERENCES users(id),
    FOREIGN KEY(mailbox_id) REFERENCES mailboxes(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_mailbox_assignments_user ON mailbox_assignments(user_id, mailbox_id);
CREATE INDEX IF NOT EXISTS idx_domain_admin_assignments_user ON domain_admin_assignments(user_id, domain_id);
