-- ============================================================
-- 1. 创建产品变体表
-- ============================================================
CREATE TABLE IF NOT EXISTS product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku VARCHAR(255) NOT NULL,
    color_code VARCHAR(100),
    color VARCHAR(100),
    color_name VARCHAR(255),
    size VARCHAR(50),
    price DOUBLE PRECISION,  -- 直接使用 DOUBLE PRECISION
    stock INTEGER DEFAULT 0,
    weight VARCHAR(100),
    package_info TEXT,
    is_default BOOLEAN DEFAULT false,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(product_id, sku),
    UNIQUE(product_id, color, size)
);
CREATE INDEX IF NOT EXISTS idx_variants_product_id ON product_variants(product_id);

-- ============================================================
-- 2. 创建或补全产品图片表（包含所有原始列 + 新列）
-- ============================================================
CREATE TABLE IF NOT EXISTS product_images (
    id SERIAL PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES product_variants(id) ON DELETE SET NULL,
    url VARCHAR(500) NOT NULL,
    name VARCHAR(255),
    original_name VARCHAR(255),
    color_code VARCHAR(10),
    image_type VARCHAR(20) DEFAULT 'other',
    file_size BIGINT,
    width INTEGER,
    height INTEGER,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 如果表已存在（旧版本），用 ALTER 补充可能缺失的列（幂等）
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS variant_id UUID REFERENCES product_variants(id) ON DELETE SET NULL;
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS color_code VARCHAR(10);
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS image_type VARCHAR(20) DEFAULT 'other';
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS original_name VARCHAR(255);
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS file_size BIGINT;
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS width INTEGER;
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS height INTEGER;

-- ============================================================
-- 3. 创建索引（兼容旧表，避免重复报错）
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_product_images_product_id ON product_images(product_id);
CREATE INDEX IF NOT EXISTS idx_product_images_variant_id ON product_images(variant_id);
CREATE INDEX IF NOT EXISTS idx_product_images_color_code ON product_images(color_code);

-- 修改 price 字段类型（幂等，支持已有表）
ALTER TABLE product_variants 
ALTER COLUMN price TYPE DOUBLE PRECISION USING price::DOUBLE PRECISION;