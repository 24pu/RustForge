-- 插入模版管理相关权限
INSERT INTO permissions (name, description, module) VALUES
    ('template:list', '查看模版列表', 'template'),
    ('template:create', '创建模版', 'template'),
    ('template:edit', '编辑模版', 'template'),
    ('template:delete', '删除模版', 'template')
ON CONFLICT (name) DO NOTHING;

-- 为管理员角色分配以上所有模版权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.name = 'admin' AND p.module = 'template'
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO permissions (name, description, module) 
VALUES ('template:manage', '模版管理全部权限', 'template')
ON CONFLICT (name) DO NOTHING;

-- 为 admin 分配该权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.name = 'admin' AND p.name = 'template:manage'
ON CONFLICT (role_id, permission_id) DO NOTHING;