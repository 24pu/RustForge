-- 创建权限表
CREATE TABLE IF NOT EXISTS permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    module VARCHAR(50)
);

-- 创建角色权限关联表
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id INTEGER REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- 初始化完整权限数据（合并旧库所有权限）
INSERT INTO permissions (name, description, module) VALUES
    ('dashboard:view', '查看仪表盘', 'dashboard'),
    ('content:list', '列出内容', 'content'),
    ('content:create', '创建内容', 'content'),
    ('content:edit', '编辑内容', 'content'),
    ('content:delete', '删除内容', 'content'),
    ('user:list', '列出用户', 'user'),
    ('user:create', '创建用户', 'user'),
    ('user:edit', '编辑用户', 'user'),
    ('user:delete', '删除用户', 'user'),
    ('role:list', '列出角色', 'role'),
    ('role:create', '创建角色', 'role'),
    ('role:edit', '编辑角色', 'role'),
    ('role:delete', '删除角色', 'role'),
    ('role:assign', '分配角色', 'role'),
    ('config:view', '查看系统设置', 'config'),
    ('config:edit', '修改系统设置', 'config'),
    ('category:list', '查看分类树', 'category'),
    ('category:create', '创建分类', 'category'),
    ('category:edit', '编辑分类', 'category'),
    ('category:delete', '删除分类', 'category'),
    ('media:list', '查看媒体库', 'media'),
    ('media:upload', '上传文件', 'media'),
    ('media:delete', '删除文件', 'media'),
    ('folder:list', '查看文件夹', 'media'),
    ('folder:create', '创建文件夹', 'media'),
    ('folder:edit', '重命名文件夹', 'media'),
    ('folder:delete', '删除文件夹', 'media'),
    ('theme:list', '查看主题列表', 'theme'),
    ('theme:activate', '切换主题', 'theme'),
    ('theme:edit', '编辑主题模板', 'theme'),
    ('plugin:list', '查看插件列表', 'plugin'),
    ('plugin:install', '安装插件', 'plugin'),
    ('plugin:uninstall', '卸载插件', 'plugin'),
    ('plugin:manage', '启用/禁用插件', 'plugin')
ON CONFLICT (name) DO NOTHING;

-- 清空原有角色权限关系，重新分配
DELETE FROM role_permissions;

-- 管理员拥有所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p WHERE r.name = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 编辑员：内容相关权限 + 仪表盘 + 媒体查看上传 + 分类列表 + 主题查看
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'editor' AND p.name IN (
    'dashboard:view',
    'content:list', 'content:create', 'content:edit', 'content:delete',
    'media:list', 'media:upload',
    'category:list',
    'theme:list'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 查看者：只读
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'viewer' AND p.name IN ('dashboard:view', 'content:list', 'media:list', 'category:list', 'theme:list')
ON CONFLICT (role_id, permission_id) DO NOTHING;