CREATE TABLE IF NOT EXISTS settings (
    id TEXT DEFAULT 'DEFAULT_SETTINGS' NOT NULL PRIMARY KEY,
    encrypted_global_api_key TEXT NOT NULL
);

INSERT INTO settings (encrypted_global_api_key)
VALUES ('ba6863ea82a0d8896284e48429cdb9fe6f14f742e5fb36bd9f9f0a2d5a86f436');