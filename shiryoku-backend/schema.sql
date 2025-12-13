DROP TABLE IF EXISTS logs;
CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tracking_id TEXT,
    timestamp TEXT,
    ip TEXT,
    country TEXT,
    city TEXT,
    user_agent TEXT,
    timezone TEXT
);

DROP TABLE IF EXISTS scheduled_emails;
CREATE TABLE scheduled_emails (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient TEXT NOT NULL,
    subject TEXT NOT NULL,
    html_body TEXT NOT NULL,
    plain_body TEXT NOT NULL,
    scheduled_at TEXT NOT NULL,
    smtp_username TEXT NOT NULL,
    smtp_password TEXT NOT NULL,
    sender_name TEXT,
    status TEXT DEFAULT 'pending',
    created_at TEXT DEFAULT (datetime('now'))
);

DROP TABLE IF EXISTS attachments;
CREATE TABLE attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email_id INTEGER NOT NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    data TEXT NOT NULL,
    is_inline BOOLEAN DEFAULT 0,
    cid TEXT,
    FOREIGN KEY(email_id) REFERENCES scheduled_emails(id) ON DELETE CASCADE
);
