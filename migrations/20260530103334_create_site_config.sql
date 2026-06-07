-- Add migration script here
-- 创建站点配置表
CREATE TABLE IF NOT EXISTS site_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 插入默认配置
INSERT INTO site_config (key, value) VALUES
    ('site_name', 'RustForge'),
    ('default_per_page', '20'),
    ('theme_color', 'blue')
ON CONFLICT (key) DO NOTHING;