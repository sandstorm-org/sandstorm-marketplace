CREATE TABLE branding_settings (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    revision INTEGER NOT NULL DEFAULT 0,
    settings_json TEXT
);
INSERT INTO branding_settings(singleton) VALUES(1);
