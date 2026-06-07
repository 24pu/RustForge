-- 分类增加展示类型（如果列不存在则添加）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'categories' AND column_name = 'display_type'
    ) THEN
        ALTER TABLE categories ADD COLUMN display_type VARCHAR(20) NOT NULL DEFAULT 'list';
    END IF;
END $$;
COMMENT ON COLUMN categories.display_type IS 'list/gallery/page';

-- 内容增加封面图字段（如果列不存在则添加）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'contents' AND column_name = 'cover_image'
    ) THEN
        ALTER TABLE contents ADD COLUMN cover_image VARCHAR(500);
    END IF;
END $$;