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
