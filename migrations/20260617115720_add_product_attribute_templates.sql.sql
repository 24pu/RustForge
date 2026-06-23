-- Add migration script here

-- ============================================================
-- 动态属性模板表 (可选，对应 amatemps)
-- 如果不使用模板，可以将自定义属性存入 products 的 JSONB 字段
-- ============================================================
DROP TABLE IF EXISTS product_attribute_templates CASCADE;
CREATE TABLE IF NOT EXISTS product_attribute_templates (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,           -- 属性名 (如 "颜色", "材质")
    title VARCHAR(255),
    value  TEXT,       -- 显示标题
    is_used BOOLEAN DEFAULT true,
    user_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


-- 1. 修正 product_attribute_values（删除重建）
DROP TABLE IF EXISTS product_attribute_values CASCADE;
CREATE TABLE product_attribute_values (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    attribute_template_id INTEGER NOT NULL REFERENCES product_attribute_templates(id) ON DELETE CASCADE,
    value VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (product_id, attribute_template_id)
);

-- 2. 分组表
DROP TABLE IF EXISTS attribute_groups CASCADE;
CREATE TABLE IF NOT EXISTS attribute_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    is_used BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
DROP TABLE IF EXISTS group_attribute_template_relations CASCADE;
-- 3. 分组-属性模板关联表
CREATE TABLE IF NOT EXISTS group_attribute_template_relations (
    group_id INTEGER NOT NULL REFERENCES attribute_groups(id) ON DELETE CASCADE,
    attribute_template_id INTEGER NOT NULL REFERENCES product_attribute_templates(id) ON DELETE CASCADE,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, attribute_template_id)
);

-- 4. 产品选择分组（方案A：修改产品表）
ALTER TABLE products ADD COLUMN IF NOT EXISTS attribute_group_id INTEGER REFERENCES attribute_groups(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_products_attribute_group_id ON products(attribute_group_id);

-- 5. 索引
CREATE INDEX IF NOT EXISTS idx_group_attribute_relations_group ON group_attribute_template_relations(group_id);
CREATE INDEX IF NOT EXISTS idx_group_attribute_relations_template ON group_attribute_template_relations(attribute_template_id);
