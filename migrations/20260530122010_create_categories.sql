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

-- 1. 顶级分类：音乐
INSERT INTO categories (name, slug, description, parent_id) VALUES
    ('乐谱', 'score', '音乐相关分类', NULL)
ON CONFLICT (slug) DO NOTHING;

-- 2. 子分类（父级为音乐）
INSERT INTO categories (name, slug, description, parent_id) VALUES
    ('古典音乐', 'classical', '西方古典音乐作品与理论', (SELECT id FROM categories WHERE slug = 'score')),
    ('流行音乐', 'pop', '现代流行、摇滚、电子等', (SELECT id FROM categories WHERE slug = 'score')),
    ('爵士音乐', 'jazz', '爵士乐风格与即兴', (SELECT id FROM categories WHERE slug = 'score')),
    ('民族音乐', 'folk', '世界民族音乐、中国传统音乐', (SELECT id FROM categories WHERE slug = 'score')),
    ('乐理与视唱', 'music-theory', '音阶、调式、节奏、视唱练耳', (SELECT id FROM categories WHERE slug = 'score')),
    ('和声与作曲', 'harmony-composition', '和声学、曲式分析、作曲技术', (SELECT id FROM categories WHERE slug = 'score'))
ON CONFLICT (slug) DO NOTHING;



