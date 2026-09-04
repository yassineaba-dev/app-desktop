CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    full_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('admin', 'user', 'viewer')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT DEFAULT NULL,
    sync_version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS incoming (
    id TEXT PRIMARY KEY NOT NULL,
    registration_number TEXT NOT NULL,
    correspondence_number TEXT,
    date TEXT NOT NULL,
    subject TEXT NOT NULL,
    sender TEXT NOT NULL,
    destination_service TEXT NOT NULL,
    source TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT DEFAULT NULL,
    created_by TEXT REFERENCES users(id),
    sync_version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS outgoing (
    id TEXT PRIMARY KEY NOT NULL,
    registration_number TEXT NOT NULL,
    correspondence_number TEXT,
    date TEXT NOT NULL,
    subject TEXT NOT NULL,
    recipient TEXT NOT NULL,
    destination_service TEXT NOT NULL,
    source TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT DEFAULT NULL,
    created_by TEXT REFERENCES users(id),
    sync_version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT REFERENCES users(id),
    action TEXT NOT NULL CHECK(action IN ('create', 'update', 'delete')),
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    metadata TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS sync_metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO sync_metadata (key, value) VALUES
    ('last_pushed_version', '0'),
    ('last_pulled_version', '0'),
    ('last_sync_at', ''),
    ('sync_status', 'offline');
