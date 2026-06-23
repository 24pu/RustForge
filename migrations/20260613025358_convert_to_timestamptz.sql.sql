-- Add migration script here
-- 迁移：将 TIMESTAMP 转换为 TIMESTAMPTZ

-- 1. 产品分类表
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'product_categories' AND column_name = 'created_at' 
               AND data_type = 'timestamp without time zone') THEN
        ALTER TABLE product_categories ALTER COLUMN created_at TYPE TIMESTAMPTZ;
        ALTER TABLE product_categories ALTER COLUMN updated_at TYPE TIMESTAMPTZ;
    END IF;
END $$;

-- 2. 产品表
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'products' AND column_name = 'created_at' 
               AND data_type = 'timestamp without time zone') THEN
        ALTER TABLE products ALTER COLUMN created_at TYPE TIMESTAMPTZ;
        ALTER TABLE products ALTER COLUMN updated_at TYPE TIMESTAMPTZ;
    END IF;
END $$;

-- 3. 内容表
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'contents' AND column_name = 'created_at' 
               AND data_type = 'timestamp without time zone') THEN
        ALTER TABLE contents ALTER COLUMN created_at TYPE TIMESTAMPTZ;
        ALTER TABLE contents ALTER COLUMN updated_at TYPE TIMESTAMPTZ;
    END IF;
END $$;

-- 4. 分类表
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'categories' AND column_name = 'created_at' 
               AND data_type = 'timestamp without time zone') THEN
        ALTER TABLE categories ALTER COLUMN created_at TYPE TIMESTAMPTZ;
        ALTER TABLE categories ALTER COLUMN updated_at TYPE TIMESTAMPTZ;
    END IF;
END $$;