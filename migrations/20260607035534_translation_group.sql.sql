-- 为已有内容生成翻译组（如果为空）
UPDATE contents SET translation_group = gen_random_uuid() WHERE translation_group IS NULL;

-- 删除旧的 slug 唯一约束（如果存在）
ALTER TABLE contents DROP CONSTRAINT IF EXISTS contents_slug_key;

-- 添加新的组合唯一约束（slug + lang），仅当不存在时添加
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'contents_slug_lang_key' 
        AND conrelid = 'contents'::regclass
    ) THEN
        ALTER TABLE contents ADD CONSTRAINT contents_slug_lang_key UNIQUE (slug, lang);
    END IF;
END $$;