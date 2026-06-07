-- 添加默认语言配置（如果不存在）
INSERT INTO site_config (key, value) VALUES
    ('default_lang', 'zh'),
    ('supported_langs', 'zh,en')
ON CONFLICT (key) DO NOTHING;

-- 为 contents 表添加语言字段（如果不存在）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'contents' AND column_name = 'lang'
    ) THEN
        ALTER TABLE contents ADD COLUMN lang VARCHAR(10) NOT NULL DEFAULT 'zh';
    END IF;
END $$;

-- 为 contents 表添加翻译组字段（如果不存在）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'contents' AND column_name = 'translation_group'
    ) THEN
        ALTER TABLE contents ADD COLUMN translation_group UUID;
        -- 仅为新添加的列设置默认值，不影响已有数据
        UPDATE contents SET translation_group = gen_random_uuid() WHERE translation_group IS NULL;
    END IF;
END $$;

-- 如果列已存在但仍有空值，补全翻译组（安全操作）
UPDATE contents SET translation_group = gen_random_uuid() WHERE translation_group IS NULL;

-- 创建索引（如果不存在）
CREATE INDEX IF NOT EXISTS idx_contents_lang ON contents(lang);
CREATE INDEX IF NOT EXISTS idx_contents_slug_lang ON contents(slug, lang);
CREATE INDEX IF NOT EXISTS idx_contents_translation_group ON contents(translation_group);