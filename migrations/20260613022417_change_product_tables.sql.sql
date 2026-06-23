-- ============================================================
-- 迁移脚本（幂等版本）：可重复执行，不会删除已有数据
-- ============================================================

-- 1. 产品分类表
CREATE TABLE IF NOT EXISTS product_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER REFERENCES product_categories(id) ON DELETE SET NULL,
    sort INTEGER DEFAULT 0,
    show_in_nav BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. 产品主表
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY,
    slug VARCHAR(255) NOT NULL UNIQUE,
    lang VARCHAR(10) DEFAULT 'zh',
    translation_group UUID,
    name VARCHAR(255) NOT NULL,
    dname VARCHAR(255),
    fullname VARCHAR(255),
    brand VARCHAR(255),
    cover_image TEXT,
    summary TEXT,
    description TEXT,
    keywords TEXT,
    points TEXT,
    dnote TEXT,
    csize VARCHAR(100),
    sku VARCHAR(100),
    ussize VARCHAR(100),
    asize VARCHAR(100),
    fabric_type VARCHAR(100),
    price VARCHAR(50),
    stock VARCHAR(50),
    package VARCHAR(255),
    weight VARCHAR(50),
    published BOOLEAN DEFAULT false,
    user_id UUID,
    size_list TEXT,
    color_list TEXT,
    color_names TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. 产品变体表
CREATE TABLE IF NOT EXISTS product_variants (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku VARCHAR(255) NOT NULL,
    color VARCHAR(50),
    color_code VARCHAR(50),
    color_name VARCHAR(100),
    size VARCHAR(50),
    price DOUBLE PRECISION,
    stock INTEGER DEFAULT 0,
    weight VARCHAR(50),
    package_info TEXT,
    is_default BOOLEAN DEFAULT false,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 4. 产品图片表（含新增 mime_type 字段）
CREATE TABLE IF NOT EXISTS product_images (
    id SERIAL PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES product_variants(id) ON DELETE CASCADE,
    url VARCHAR(500) NOT NULL,
    name VARCHAR(255),
    original_name VARCHAR(255),
    color_code VARCHAR(50),
    image_type VARCHAR(50) DEFAULT 'main',
    file_size BIGINT,
    width INTEGER,
    height INTEGER,
    sort_order INTEGER DEFAULT 0,
    mime_type VARCHAR(100),           -- 新增字段
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 5. 产品-分类关联表
CREATE TABLE IF NOT EXISTS product_category_relations (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES product_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, category_id)
);

-- ============================================================
-- 对已存在的表添加可能缺失的列（幂等补丁）
-- ============================================================

-- 确保 product_images 表有 mime_type 列（如果表已存在且缺少该列）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'product_images' AND column_name = 'mime_type'
    ) THEN
        ALTER TABLE product_images ADD COLUMN mime_type VARCHAR(100);
    END IF;
END $$;

-- ============================================================
-- 索引（幂等创建）
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_products_slug ON products(slug);
CREATE INDEX IF NOT EXISTS idx_products_published ON products(published);
CREATE INDEX IF NOT EXISTS idx_products_translation_group ON products(translation_group);
CREATE INDEX IF NOT EXISTS idx_product_variants_product_id ON product_variants(product_id);
CREATE INDEX IF NOT EXISTS idx_product_variants_sku ON product_variants(sku);
CREATE INDEX IF NOT EXISTS idx_product_images_product_id ON product_images(product_id);
CREATE INDEX IF NOT EXISTS idx_product_images_variant_id ON product_images(variant_id);
CREATE INDEX IF NOT EXISTS idx_product_categories_parent_id ON product_categories(parent_id);