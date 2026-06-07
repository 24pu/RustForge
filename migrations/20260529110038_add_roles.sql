CREATE TABLE IF NOT EXISTS roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT
);

INSERT INTO roles (name, description) VALUES
    ('admin', '系统管理员，拥有所有权限'),
    ('editor', '内容编辑，可管理内容'),
    ('viewer', '只读用户，仅可查看')
ON CONFLICT (name) DO NOTHING;

