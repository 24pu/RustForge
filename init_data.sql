-- ========== 清理旧数据 ==========
DELETE FROM content_categories;
DELETE FROM contents;
DELETE FROM categories;

-- ========== 站点基本配置 ==========
INSERT INTO site_config (key, value) VALUES
    ('site_name', 'RustForge'),
    ('seo_title', 'RustForge - 高性能企业级网站系统'),
    ('seo_description', '基于 Rust 构建的现代企业网站，提供内容管理、插件系统、主题定制等功能。'),
    ('seo_keywords', 'Rust, CMS, 企业网站, Axum, PostgreSQL')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;

-- ========== 创建丰富的分类体系 ==========
-- 顶级分类
INSERT INTO categories (name, slug, description, parent_id, display_type, show_in_nav, sort) VALUES
    ('产品中心', 'products', '我们的核心产品与解决方案', NULL, 'list', true, 1),
    ('新闻动态', 'news', '公司新闻、技术分享与行业观察', NULL, 'list', true, 2),
    ('技术博客', 'blog', '深入技术探讨与实践教程', NULL, 'list', true, 3),
    ('关于我们', 'about', '了解 RustForge 的故事与团队', NULL, 'page', true, 4),
    ('服务支持', 'support', '帮助文档与常见问题', NULL, 'list', true, 5)
ON CONFLICT (slug) DO NOTHING;

-- 子分类
INSERT INTO categories (name, slug, description, parent_id, display_type, show_in_nav, sort) VALUES
    ('云服务', 'cloud', '云计算与基础设施', (SELECT id FROM categories WHERE slug = 'products'), 'gallery', true, 1),
    ('数据分析', 'data-analytics', '大数据分析与 AI 解决方案', (SELECT id FROM categories WHERE slug = 'products'), 'list', true, 2),
    ('安全产品', 'security', '企业级安全防护', (SELECT id FROM categories WHERE slug = 'products'), 'list', true, 3),
    ('公司新闻', 'company-news', 'RustForge 最新动态', (SELECT id FROM categories WHERE slug = 'news'), 'list', true, 1),
    ('行业洞察', 'industry-insights', '技术趋势与市场分析', (SELECT id FROM categories WHERE slug = 'news'), 'list', true, 2),
    ('Rust 实践', 'rust-tutorials', 'Rust 语言实战经验', (SELECT id FROM categories WHERE slug = 'blog'), 'list', true, 1),
    ('Web 开发', 'web-dev', '前后端全栈开发指南', (SELECT id FROM categories WHERE slug = 'blog'), 'list', true, 2),
    ('常见问题', 'faq', '常见问题与解答', (SELECT id FROM categories WHERE slug = 'support'), 'list', true, 1),
    ('API 文档', 'api-docs', '开发者接口文档', (SELECT id FROM categories WHERE slug = 'support'), 'page', true, 2)
ON CONFLICT (slug) DO NOTHING;

-- ========== 插入多篇示例内容 ==========
-- 产品中心
INSERT INTO contents (slug, title, body, published, cover_image) VALUES
    ('cloud-intro', '企业级云原生解决方案', 
     '# 企业级云原生解决方案

我们基于 Kubernetes 和 Rust 技术栈构建高可用、低延迟的云服务，助力企业数字化转型。

## 核心特性
- **弹性伸缩**：自动扩缩容，从容应对流量高峰
- **安全可靠**：内存安全 + 类型安全，避免常见漏洞
- **成本优化**：按需付费，资源利用率提升 40%

[联系我们](/contact) 获取免费试用。', true, NULL),
    ('big-data', '大数据分析平台', 
     '# 大数据分析平台

采用 Rust 编写的高性能数据处理引擎，支持实时流式计算与批量处理。

## 主要能力
- 实时数据管道
- SQL 分析接口
- 机器学习模型集成
- 可视化看板

[查看案例](/case-studies)', true, NULL),
    ('security-suite', '全方位安全防护', 
     '# 全方位安全防护

基于零信任架构的安全产品套件，包括 Web 应用防火墙、DDoS 防护、入侵检测等。

## 亮点
- 规则引擎热更新
- 极低性能开销
- 开源核心组件

[阅读白皮书](/whitepaper)', true, NULL)
ON CONFLICT (slug) DO NOTHING;

-- 新闻动态
INSERT INTO contents (slug, title, body, published, cover_image) VALUES
    ('rustforge-launch', 'RustForge 正式发布 v1.0', 
     '经过 12 个月的开发，我们自豪地宣布 RustForge v1.0 正式发布！这是一个基于 Rust + Axum 的企业级网站系统，支持插件、主题和 RBAC 权限。立即访问 [GitHub](https://github.com) 查看源码。', true, NULL),
    ('partnership-cloud', '与 CloudBase 达成战略合作', 
     'RustForge 与 CloudBase 建立深度合作关系，未来将共同提供云原生产品和服务。双方将在技术集成、市场推广等方面展开合作。', true, NULL),
    ('webinar-rust', '线上研讨会：Rust 在企业应用中的实践', 
     '我们将于下周三举办线上研讨会，分享 Rust 在金融、电商等领域的最佳实践。欢迎报名参加！', true, NULL)
ON CONFLICT (slug) DO NOTHING;

-- 技术博客
INSERT INTO contents (slug, title, body, published, cover_image) VALUES
    ('async-rust', '异步 Rust 实战：从入门到生产', 
     '本文详细介绍 Rust 异步编程模型，包括 Future、async/await、Tokio 运行时等，并通过一个完整的 HTTP 服务器示例进行演示。', true, NULL),
    ('tera-templates', '使用 Tera 模板引擎构建优雅的前端', 
     'Tera 是一个受 Jinja2 启发的 Rust 模板引擎，语法简单，功能强大。本文展示如何在 Axum 中集成 Tera 并创建可重用的模板组件。', true, NULL),
    ('sqlx-offline', 'SQLx 离线编译模式的原理与使用', 
     'SQLx 提供离线模式，可在编译期验证 SQL 语句，无需连接数据库。本文解释其原理，并演示如何在 CI/CD 环境中使用。', true, NULL)
ON CONFLICT (slug) DO NOTHING;

-- 关于我们
INSERT INTO contents (slug, title, body, published, cover_image) VALUES
    ('about-us', '关于 RustForge', 
     'RustForge 由一支热爱 Rust 语言的团队创建，致力于用 Rust 构建安全、快速、可扩展的 Web 应用基础设施。我们的使命是**让 Rust 走进企业，让开发更简单**。', true, NULL)
ON CONFLICT (slug) DO NOTHING;

-- 服务支持
INSERT INTO contents (slug, title, body, published, cover_image) VALUES
    ('faq-content', '常见问题 FAQ', 
     '## 如何安装 RustForge？
请参考 [安装文档](/docs/install)。

## 支持哪些数据库？
目前仅支持 PostgreSQL。

## 如何开发插件？
查看 [插件开发指南](/docs/plugin)。', true, NULL),
    ('api-docs', 'API 接口文档', 
     '本文档描述 RustForge 提供的 RESTful API，包括认证、内容管理、用户管理等接口。详细内容可参考 [API 参考](/api-docs)。', true, NULL)
ON CONFLICT (slug) DO NOTHING;

-- ========== 关联内容与分类 ==========
INSERT INTO content_categories (content_id, category_id) VALUES
    ((SELECT id FROM contents WHERE slug = 'cloud-intro'), (SELECT id FROM categories WHERE slug = 'cloud')),
    ((SELECT id FROM contents WHERE slug = 'big-data'), (SELECT id FROM categories WHERE slug = 'data-analytics')),
    ((SELECT id FROM contents WHERE slug = 'security-suite'), (SELECT id FROM categories WHERE slug = 'security')),
    ((SELECT id FROM contents WHERE slug = 'rustforge-launch'), (SELECT id FROM categories WHERE slug = 'company-news')),
    ((SELECT id FROM contents WHERE slug = 'partnership-cloud'), (SELECT id FROM categories WHERE slug = 'company-news')),
    ((SELECT id FROM contents WHERE slug = 'webinar-rust'), (SELECT id FROM categories WHERE slug = 'industry-insights')),
    ((SELECT id FROM contents WHERE slug = 'async-rust'), (SELECT id FROM categories WHERE slug = 'rust-tutorials')),
    ((SELECT id FROM contents WHERE slug = 'tera-templates'), (SELECT id FROM categories WHERE slug = 'web-dev')),
    ((SELECT id FROM contents WHERE slug = 'sqlx-offline'), (SELECT id FROM categories WHERE slug = 'rust-tutorials')),
    ((SELECT id FROM contents WHERE slug = 'about-us'), (SELECT id FROM categories WHERE slug = 'about')),
    ((SELECT id FROM contents WHERE slug = 'faq-content'), (SELECT id FROM categories WHERE slug = 'faq')),
    ((SELECT id FROM contents WHERE slug = 'api-docs'), (SELECT id FROM categories WHERE slug = 'api-docs'))
ON CONFLICT DO NOTHING;

-- 确保管理员拥有所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;