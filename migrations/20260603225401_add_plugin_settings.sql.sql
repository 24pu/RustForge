CREATE TABLE IF NOT EXISTS plugin_settings (
    plugin_name TEXT PRIMARY KEY,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb
);