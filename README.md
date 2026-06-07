# RustForge

基于 Rust + Axum 的企业级网站系统，支持 Wasm 插件、Tera 主题、RBAC 权限、多语言和 Docker 部署。

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15+-blue.svg)](https://www.postgresql.org)

## ✨ 特性

- 🚀 **高性能**：Rust + Axum + SQLx，异步 I/O，无 GC
- 🔌 **插件系统**：基于 WebAssembly (Wasmtime)，安全动态扩展
- 🎨 **主题系统**：Tera 模板引擎，支持多主题切换
- 🔐 **RBAC 权限**：用户-角色-权限细粒度控制
- 🌐 **多语言**：自动加载主程序和插件语言包
- 📝 **内容管理**：Markdown 编辑器、多分类、封面图、SEO
- 🖼️ **媒体库**：文件上传、文件夹管理、自动缩略图
- 🐳 **一键部署**：Docker 支持，自带安装向导

## 🚀 快速开始

### 环境要求

- Rust 1.75+
- PostgreSQL 15+
- Docker（可选）

### 本地开发

```bash
# 克隆项目
git clone https://github.com/24pu/RustForge.git
cd RustForge

# 配置环境
cp .env.example .env  # 编辑数据库连接信息

# 运行数据库迁移
sqlx migrate run

# 启动服务
cargo run

访问 http://localhost:3000，首次访问会进入安装向导。

Docker 部署
bash
# 构建镜像
docker build -t rustforge .

# 运行容器
docker run -d --name rustforge -p 3000:3000 \
  -e DATABASE_URL="postgresql://user:pass@host:5432/rustforge" \
  rustforge
📁 项目结构
text
rustforge/
├── src/                    # 源代码
│   ├── presentation/       # 表现层（路由、处理器、中间件）
│   ├── core/               # 核心业务接口
│   └── infrastructure/     # 基础设施（数据库、认证、插件运行时）
├── plugins/                # 插件目录
├── themes/                 # 主题目录
├── locales/                # 多语言翻译文件
├── migrations/             # 数据库迁移
└── frontend/dist/          # 管理后台静态文件
📚 文档
插件开发指南 http://rustforge.24pu.com/plugins/docs/static/plugin-dev-guide.html

主题开发指南 http://rustforge.24pu.com/plugins/docs/static/theme-dev-guide.html

主程序开发文档 http://rustforge.24pu.com/plugins/docs/static/main-program-dev-guide.html

🛠️ 技术栈
技术	用途
Rust	后端语言
Axum	Web 框架
SQLx + PostgreSQL	数据库
Tera	模板引擎
Wasmtime	插件运行时
Tailwind CSS	前端样式
📄 许可证
本项目采用 MIT License。

🔗 链接
官方网站：http://rustforge.24pu.com

GitHub：https://github.com/24pu/RustForge

