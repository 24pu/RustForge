-- ============================================================
-- 站点配置表（幂等版本 - 可重复执行，不丢数据）
-- ============================================================

-- 1. 创建表（如果不存在）
CREATE TABLE IF NOT EXISTS site_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. 插入默认配置（如果不存在）
INSERT INTO site_config (key, value) VALUES
    -- 网站设置
    ('site_name', 'RustForge'),
    ('default_per_page', '20'),
    ('theme_color', 'blue'),
    
    -- SEO 设置
    ('seo_title', ''),
    ('seo_description', ''),
    ('seo_keywords', ''),
    ('site_url', 'http://yoursite.com'),
    
    -- 图片/Logo 设置
    ('logo_url', ''),
    ('favicon_url', ''),
    
    -- 文件上传设置
    ('allowed_file_types', 'jpg,jpeg,png,gif,webp,mp4,mp3,pdf,doc,docx,xls,xlsx'),
    ('max_file_size_mb', '10'),
    
    -- 产品设置
    ('product_allowed_image_types', 'jpg,jpeg,png,gif,webp'),
    ('product_max_image_size_mb', '5'),
    ('product_max_images_count', '20'),
    ('product_auto_thumbnail', 'true'),
    ('product_thumbnail_width', '200'),
    ('product_thumbnail_height', '200'),
    ('product_size_inch', 'false')
ON CONFLICT (key) DO NOTHING;

-- 3. 创建更新时间的触发器
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS update_site_config_updated_at ON site_config;
CREATE TRIGGER update_site_config_updated_at
    BEFORE UPDATE ON site_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();