-- ============================================================
-- 完整初始化数据（包含多语言支持）- 3大类8小类
-- ============================================================

-- ========== 清理旧数据 ==========
DELETE FROM content_categories;
DELETE FROM contents;
DELETE FROM categories;

-- ========== 确保表结构包含多语言字段 ==========
ALTER TABLE contents ADD COLUMN IF NOT EXISTS lang VARCHAR(10) DEFAULT 'zh';
ALTER TABLE contents ADD COLUMN IF NOT EXISTS translation_group UUID;

-- ========== 站点基本配置 ==========
INSERT INTO site_config (key, value) VALUES
    ('site_name', 'RustForge'),
    ('seo_title', 'RustForge - 高性能企业级网站系统'),
    ('seo_description', '基于 Rust 构建的现代企业网站，提供内容管理、插件系统、主题定制等功能。'),
    ('seo_keywords', 'Rust, CMS, 企业网站, Axum, PostgreSQL')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;

-- ========== 创建内容分类体系（3大分类 + 8小分类） ==========

-- 1. 三大顶级分类
INSERT INTO categories (name, slug, description, parent_id, display_type, show_in_nav, sort) VALUES
    ('关于我们', 'about', '了解我们的故事与团队', NULL, 'page', true, 1),
    ('法律声明', 'legal', '用户协议、隐私政策等法律文件', NULL, 'page', true, 2),
    ('服务支持', 'support', '联系我们与帮助中心', NULL, 'list', true, 3)
ON CONFLICT (slug) DO NOTHING;

-- 2. 子分类：关于我们
INSERT INTO categories (name, slug, description, parent_id, display_type, show_in_nav, sort) VALUES
    ('公司介绍', 'company', 'RustForge 公司简介', (SELECT id FROM categories WHERE slug = 'about'), 'page', true, 1),
    ('发展历程', 'history', '公司发展历程', (SELECT id FROM categories WHERE slug = 'about'), 'page', true, 2),
    ('团队介绍', 'team', '核心团队成员', (SELECT id FROM categories WHERE slug = 'about'), 'page', true, 3)
ON CONFLICT (slug) DO NOTHING;

-- 3. 子分类：法律声明
INSERT INTO categories (name, slug, description, parent_id, display_type, show_in_nav, sort) VALUES
    ('用户协议', 'terms', '注册协议与使用条款', (SELECT id FROM categories WHERE slug = 'legal'), 'page', true, 1),
    ('隐私政策', 'privacy', '个人信息保护说明', (SELECT id FROM categories WHERE slug = 'legal'), 'page', true, 2),
    ('免责声明', 'disclaimer', '网站内容免责声明', (SELECT id FROM categories WHERE slug = 'legal'), 'page', true, 3),
    ('版权声明', 'copyright', '版权归属与转载政策', (SELECT id FROM categories WHERE slug = 'legal'), 'page', true, 4),
    ('服务条款', 'service-terms', '服务范围与用户义务', (SELECT id FROM categories WHERE slug = 'legal'), 'page', true, 5)
ON CONFLICT (slug) DO NOTHING;

-- 4. 子分类：服务支持
INSERT INTO categories (name, slug, description, parent_id, display_type, show_in_nav, sort) VALUES
    ('联系我们', 'contact', '获取联系方式与反馈渠道', (SELECT id FROM categories WHERE slug = 'support'), 'page', true, 1),
    ('帮助中心', 'help', '常见问题与使用指南', (SELECT id FROM categories WHERE slug = 'support'), 'list', true, 2)
ON CONFLICT (slug) DO NOTHING;

-- ========== 插入内容页面（带多语言字段） ==========
INSERT INTO contents (slug, lang, translation_group, title, body, published) VALUES
    
    -- ===== 关于我们 =====
    
    -- 1. 公司介绍
    ('company', 'zh', gen_random_uuid(), '公司介绍', 
     '# 公司介绍

RustForge 由一支热爱 Rust 语言的团队创建，致力于用 Rust 构建安全、快速、可扩展的 Web 应用基础设施。

## 我们的使命
**让 Rust 走进企业，让开发更简单。**

## 我们的愿景
成为企业级 Rust 应用开发的引领者，推动 Rust 生态在企业场景的落地。

## 我们的价值观
- **安全优先**：内存安全 + 类型安全，从源头杜绝漏洞
- **性能极致**：零成本抽象，充分发挥硬件性能
- **开放共赢**：拥抱开源，与社区共同成长

## 联系我们
如有合作意向，请通过 [联系我们](/contact) 页面与我们取得联系。',
     true),
    
    -- 2. 发展历程
    ('history', 'zh', gen_random_uuid(), '发展历程', 
     '# 发展历程

## 2024 年
- **1 月**：项目启动，确立技术选型（Rust + Axum + PostgreSQL）
- **3 月**：完成核心框架搭建，实现基础 CRUD 功能
- **6 月**：发布 v0.1 测试版，邀请首批内测用户
- **9 月**：重构插件系统，支持 Wasm 插件
- **12 月**：正式发布 v1.0，开源核心代码

## 2025 年
- **1 月**：上线主题市场，支持第三方主题
- **3 月**：用户数突破 1000+
- **6 月**：发布 v1.5，新增产品管理模块
- **9 月**：获得年度开源创新奖
- **12 月**：用户数突破 5000+

## 2026 年
- **持续迭代**：优化性能，丰富生态
- **国际化**：支持多语言版本
- **企业版**：推出企业级服务方案',
     true),
    
    -- 3. 团队介绍
    ('team', 'zh', gen_random_uuid(), '团队介绍', 
     '# 核心团队

RustForge 团队成员来自知名科技企业，在 Rust、分布式系统、云计算等领域拥有丰富经验。

## 创始人 & CEO
**张三**
- 前某大厂技术总监
- 10 年 + 后端开发经验
- Rust 社区活跃贡献者

## CTO
**李四**
- 分布式系统专家
- 曾主导多个高并发项目
- Rust 开源项目 Maintainer

## 核心工程师
**王五**
- 全栈开发工程师
- 擅长 Axum + React 技术栈
- 开源社区 Contributor

## 社区运营
**赵六**
- 技术社区运营专家
- 负责开发者关系与生态建设

## 加入我们
我们正在寻找热爱 Rust 的开发者！查看 [招贤纳士](/jobs) 了解详情。',
     true),
    
    -- ===== 法律声明 =====
    
    -- 4. 用户协议
    ('terms', 'zh', gen_random_uuid(), '用户协议', 
     '# 用户协议

感谢您使用 RustForge 服务。本协议是您与 RustForge 之间关于使用本服务的法律协议。

## 第一章 总则

### 1.1 协议范围
本协议适用于您访问和使用 RustForge 网站及相关服务的所有行为。

### 1.2 协议接受
使用本服务即表示您已阅读、理解并同意本协议的全部内容。

## 第二章 账号管理

### 2.1 账号注册
您需提供真实、准确的注册信息，并对账号安全性负责。

### 2.2 账号使用
账号仅限本人使用，不得转让、出借或用于非法目的。

## 第三章 用户义务

### 3.1 遵守法律法规
您在使用本服务时需遵守中华人民共和国相关法律法规。

### 3.2 内容规范
您不得发布违法、侵权、虚假或有害的信息。

## 第四章 服务条款

### 4.1 服务内容
本服务提供内容管理、插件扩展、主题定制等功能。

### 4.2 服务变更
我们保留随时调整服务内容及收费标准的权利。

### 4.3 服务终止
如您违反本协议，我们有权暂停或终止您的账号使用权。

## 第五章 免责声明

### 5.1 服务按现状提供
本服务按「现状」提供，不提供任何明示或暗示的保证。

### 5.2 不可抗力
因不可抗力导致的服务中断，我方不承担违约责任。

## 第六章 争议解决
因本协议引起的争议，双方应友好协商解决；协商不成的，提交有管辖权的人民法院诉讼解决。

## 联系方式
如有疑问，请通过 [联系我们](/contact) 页面与我们取得联系。

*更新时间：2025 年 1 月 1 日*',
     true),
    
    -- 5. 隐私政策
    ('privacy', 'zh', gen_random_uuid(), '隐私政策', 
     '# 隐私政策

RustForge 非常重视您的隐私。本政策将说明我们如何收集、使用和保护您的个人信息。

## 一、信息收集

### 1.1 您主动提供的信息
- 注册时提供的邮箱、用户名
- 使用服务时提交的内容
- 联系客服时提供的信息

### 1.2 自动收集的信息
- 访问日志（IP 地址、浏览器类型、访问时间）
- Cookie 数据
- 设备信息

## 二、信息使用

### 2.1 使用目的
- 提供并维护服务
- 个性化用户体验
- 发送通知和更新
- 改进产品和服务

### 2.2 数据保留
我们将在必要的期限内保留您的信息，或在法律要求时保留。

## 三、信息分享

### 3.1 第三方分享
我们不会向第三方出售、出租或分享您的个人信息。

## 四、数据安全

### 4.1 安全措施
我们采取行业标准的安全措施保护您的数据。

### 4.2 安全承诺
我们将持续评估和加强安全措施。

## 五、用户权利
您有权随时访问、修改和删除您的个人信息。

## 六、Cookie 使用
我们使用 Cookie 提供更好的用户体验。

## 七、联系方式
如有隐私相关问题，请联系我们：privacy@rustforge.com

*更新时间：2025 年 1 月 1 日*',
     true),
    
    -- 6. 免责声明
    ('disclaimer', 'zh', gen_random_uuid(), '免责声明', 
     '# 免责声明

欢迎访问 RustForge。本声明将明确本网站的使用规则与免责事项。

## 一、内容免责

### 1.1 信息准确性
本网站提供的内容仅供参考，我们尽力确保信息准确，但不保证完全无误。

### 1.2 用户生成内容
用户发布的内容不代表本网站立场，用户需对自身发布的内容负责。

## 二、外部链接免责
本网站可能包含指向第三方网站的链接，我们对第三方网站的内容不承担任何责任。

## 三、技术免责

### 3.1 服务中断
因网络故障、系统维护或不可抗力导致的服务中断，我们不予负责。

### 3.2 数据丢失
我们不对不可抗力造成的数据丢失承担责任。

## 四、知识产权
本网站所有内容的知识产权归 RustForge 所有。

## 五、法律适用
本声明适用中华人民共和国法律。

*如有疑问，请联系我们。*',
     true),
    
    -- 7. 版权声明
    ('copyright', 'zh', gen_random_uuid(), '版权声明', 
     '# 版权声明

## 一、版权归属

### 1.1 网站内容
本网站所有内容（包括文字、图片、音频、视频、设计、代码等）的版权归 RustForge 所有。

### 1.2 开源项目
RustForge 核心代码基于 MIT 或 Apache 2.0 许可证开源。

## 二、使用许可

### 2.1 个人使用
个人可在非商业用途下浏览、分享本网站内容，但需保留版权标识。

### 2.2 商业使用
任何商业用途需获得我们的书面授权。

## 三、禁止行为
未经授权，不得复制、修改、分发、展示或用于商业目的使用本站内容。

## 四、侵权处理

### 4.1 投诉流程
如您认为本站内容侵犯了您的权益，请提供权利证明和侵权描述。

### 4.2 处理时效
我们将在收到有效投诉后 48 小时内核实并处理。

## 五、联系方式
版权相关事宜请联系：copyright@rustforge.com',
     true),
    
    -- 8. 服务条款
    ('service-terms', 'zh', gen_random_uuid(), '服务条款', 
     '# 服务条款

## 一、服务范围

### 1.1 服务内容
RustForge 提供以下服务：
- 企业级网站搭建与管理
- 内容发布与分发
- 插件扩展与集成
- 主题定制与更换

### 1.2 适用对象
本服务适用于企业、开发者、内容创作者等合法用户。

## 二、用户权利与义务

### 2.1 用户权利
- 在服务范围内自由使用产品
- 获取产品更新和技术支持

### 2.2 用户义务
- 遵守法律法规
- 保护账号安全
- 尊重知识产权

## 三、服务标准

### 3.1 可用性承诺
年度可用性不低于 99.5%。

### 3.2 技术支持
付费用户享受专业技术支持。

## 四、服务变更与终止

### 4.1 服务变更
我们保留调整服务内容的权利，重大变更将提前公告。

### 4.2 服务终止
用户违反条款或主动申请注销可能导致服务终止。

## 五、免责与责任限制
在法律允许的最大范围内，我们对间接损失不承担责任。

*如有疑问，请联系我们。*',
     true),
    
    -- ===== 服务支持 =====
    
    -- 9. 联系我们
    ('contact', 'zh', gen_random_uuid(), '联系我们', 
     '# 联系我们

## 联系方式

### 公司地址
中国 · 某市 · 某科技园区 A 座 12 楼

### 联系电话
📞 400-888-8888

### 电子邮箱
📧 contact@rustforge.com

### 工作时间
🕐 周一至周五 9:00 - 18:00

## 在线留言
您也可以通过以下方式联系我们：

- [在线客服](/chat)
- [提交工单](/support/ticket)
- [邮件咨询](mailto:contact@rustforge.com)

## 社交媒体
- GitHub: [github.com/rustforge](https://github.com)
- 微信公众号: RustForge
- 技术社区: [rustforge.cn](https://rustforge.cn)',
     true),
    
    -- 10. 帮助中心
    ('help', 'zh', gen_random_uuid(), '帮助中心', 
     '# 帮助中心

## 快速入门
1. 注册账户
2. 配置站点信息
3. 创建内容
4. 发布上线

## 常见问题

### 如何注册账户？
点击首页右上角「注册」按钮，填写邮箱和密码即可完成注册。

### 忘记密码怎么办？
在登录页点击「忘记密码」，通过邮箱重置密码。

### 如何修改网站配置？
登录后台后，进入「系统设置」即可修改各类配置。

### 支持哪些浏览器？
支持 Chrome、Firefox、Safari、Edge 等现代浏览器。

### 如何获取技术支持？
- 查阅 [帮助中心](/help)
- 提交 [工单](/support/ticket)
- 发送邮件至 support@rustforge.com

## 更多帮助
如问题仍未解决，请联系我们的客服团队。',
     true)
ON CONFLICT (slug, lang) DO UPDATE SET body = EXCLUDED.body;

-- ========== 关联内容与分类 ==========
INSERT INTO content_categories (content_id, category_id)
SELECT c.id, cat.id 
FROM contents c 
JOIN categories cat ON 
    -- 关于我们
    (c.slug = 'company' AND cat.slug = 'company') OR
    (c.slug = 'history' AND cat.slug = 'history') OR
    (c.slug = 'team' AND cat.slug = 'team') OR
    -- 法律声明
    (c.slug = 'terms' AND cat.slug = 'terms') OR
    (c.slug = 'privacy' AND cat.slug = 'privacy') OR
    (c.slug = 'disclaimer' AND cat.slug = 'disclaimer') OR
    (c.slug = 'copyright' AND cat.slug = 'copyright') OR
    (c.slug = 'service-terms' AND cat.slug = 'service-terms') OR
    -- 服务支持
    (c.slug = 'contact' AND cat.slug = 'contact') OR
    (c.slug = 'help' AND cat.slug = 'help')
ON CONFLICT DO NOTHING;
