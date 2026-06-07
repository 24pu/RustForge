-- Add migration script here
-- 1. 创建分类表（自引用，支持无限级）
CREATE TABLE IF NOT EXISTS categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER REFERENCES categories(id) ON DELETE CASCADE,  -- 自引用
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. 创建内容-分类关联表（多对多）
CREATE TABLE IF NOT EXISTS  content_categories (
    content_id UUID REFERENCES contents(id) ON DELETE CASCADE,
    category_id INTEGER REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (content_id, category_id)
);

-- 3. 向 contents 表删除 category_id 字段（如果之前添加了单列）
ALTER TABLE contents DROP COLUMN IF EXISTS category_id;

-- 先插入顶级分类
INSERT INTO categories (name, slug, description, parent_id) VALUES
    ('产品', 'products', '公司产品介绍', NULL),
    ('新闻', 'news', '公司新闻动态', NULL),
    ('技术', 'tech', '技术文章', NULL)
ON CONFLICT (slug) DO NOTHING;

-- 再插入子分类（依赖父分类已存在）
INSERT INTO categories (name, slug, description, parent_id) VALUES
    ('前端开发', 'frontend', '前端技术', (SELECT id FROM categories WHERE slug = 'tech')),
    ('后端开发', 'backend', '后端技术', (SELECT id FROM categories WHERE slug = 'tech'))
ON CONFLICT (slug) DO NOTHING;