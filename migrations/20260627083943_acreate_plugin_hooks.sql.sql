-- ============================================================
-- 插件钩子表（幂等版本，可重复执行）
-- ============================================================

-- 创建表（如果不存在）
CREATE TABLE IF NOT EXISTS plugin_hooks (
    id SERIAL PRIMARY KEY,
    plugin_name VARCHAR(255) NOT NULL,
    hook_name VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    lang VARCHAR(10) DEFAULT 'zh',
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- 创建索引（如果不存在）
CREATE INDEX IF NOT EXISTS idx_plugin_hooks_plugin ON plugin_hooks(plugin_name);
CREATE INDEX IF NOT EXISTS idx_plugin_hooks_hook ON plugin_hooks(hook_name);
CREATE INDEX IF NOT EXISTS idx_plugin_hooks_lang ON plugin_hooks(lang);