-- ===== 示例数据：乐谱分类 + 示例文章 =====
-- 1. 顶级分类：乐谱
INSERT INTO categories (name, slug, description, parent_id)
VALUES ('乐谱', 'score', '乐谱相关分类', NULL)
ON CONFLICT (slug) DO NOTHING;

-- 2. 子分类
INSERT INTO categories (name, slug, description, parent_id)
VALUES
    ('五线谱', 'staff', '标准五线谱', (SELECT id FROM categories WHERE slug = 'score')),
    ('简谱', 'numbered-notation', '数字简谱', (SELECT id FROM categories WHERE slug = 'score')),
    ('吉他谱', 'guitar-tab', '吉他指法谱', (SELECT id FROM categories WHERE slug = 'score')),
    ('钢琴谱', 'piano-sheet', '钢琴乐谱', (SELECT id FROM categories WHERE slug = 'score')),
    ('总谱', 'full-score', '乐队总谱', (SELECT id FROM categories WHERE slug = 'score')),
    ('乐理基础', 'music-theory', '音阶、调式、节奏、视唱', (SELECT id FROM categories WHERE slug = 'score'))
ON CONFLICT (slug) DO NOTHING;

