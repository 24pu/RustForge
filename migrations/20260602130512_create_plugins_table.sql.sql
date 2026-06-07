-- 创建插件表（如果不存在）
CREATE TABLE IF NOT EXISTS plugins (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    version VARCHAR(20) NOT NULL,
    author VARCHAR(100),
    description TEXT,
    file_path VARCHAR(500) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 插入示例插件（如果名称冲突则跳过）
INSERT INTO plugins (name, version, author, description, file_path, enabled) VALUES
    ('user-center', '1.0.0', 'RustForge Team', '用户中心插件，提供个人资料等功能', 'plugins/user-center/user_center.wasm', true)
ON CONFLICT (name) DO NOTHING;