-- 创建媒体文件表
CREATE TABLE IF NOT EXISTS media_files (
    id SERIAL PRIMARY KEY,
    filename VARCHAR(255) NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    extension VARCHAR(20) NOT NULL,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 创建索引（如果不存在则创建）
CREATE INDEX IF NOT EXISTS idx_media_uploaded_by ON media_files(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_media_created_at ON media_files(created_at);

-- 为系统设置表添加媒体库相关配置（如果键已存在则跳过）
INSERT INTO site_config (key, value) VALUES
    ('allowed_file_types', 'jpg,jpeg,png,gif,mp4,mp3,pdf,doc,docx,xls,xlsx'),
    ('max_file_size_mb', '10')
ON CONFLICT (key) DO NOTHING;