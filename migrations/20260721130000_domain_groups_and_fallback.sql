-- Migration to support Domain Groups and Fallback Email

-- Add fallback_email to users table
ALTER TABLE users ADD COLUMN fallback_email TEXT;

-- Create user_domain_assignments table
CREATE TABLE IF NOT EXISTS user_domain_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    domain_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(domain_id) REFERENCES domains(id),
    UNIQUE(user_id, domain_id)
);
