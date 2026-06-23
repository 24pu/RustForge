-- Add migration script here
-- =====================================================
-- 迁移脚本：添加产品管理相关权限并赋予管理员角色
-- 版本：2025.01.01.001
-- 描述：新增产品及产品分类的 CRUD 权限，并分配给 admin 角色
-- 特性：可重复执行，不会产生重复数据或错误
-- =====================================================

BEGIN;

-- 1. 确保 admin 角色存在（如果不存在则创建）
INSERT INTO roles (name, description)
SELECT 'admin', '系统管理员，拥有所有权限'
WHERE NOT EXISTS (SELECT 1 FROM roles WHERE name = 'admin');

-- 2. 插入产品模块所需权限（如果已存在则跳过）
INSERT INTO permissions (name, description, module) VALUES
    ('product:list', '查看产品列表', 'product'),
    ('product:create', '创建产品', 'product'),
    ('product:edit', '编辑产品', 'product'),
    ('product:delete', '删除产品', 'product'),
    ('product_category:list', '查看产品分类', 'product'),
    ('product_category:create', '创建产品分类', 'product'),
    ('product_category:edit', '编辑产品分类', 'product'),
    ('product_category:delete', '删除产品分类', 'product')
ON CONFLICT (name) DO NOTHING;

-- 3. 将上述所有 product 模块的权限分配给 admin 角色
--    使用 ON CONFLICT 避免重复分配
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.module = 'product'
ON CONFLICT (role_id, permission_id) DO NOTHING;

COMMIT;