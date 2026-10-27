CREATE TABLE IF NOT EXISTS homepage_settings (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    revision INTEGER NOT NULL DEFAULT 0,
    order_json TEXT
);
INSERT OR IGNORE INTO homepage_settings(singleton) VALUES(1);
