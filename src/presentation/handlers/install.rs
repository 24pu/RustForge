

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

use crate::presentation::AppState;

#[derive(Deserialize)]
pub struct InstallRequest {
    pub email: String,
    pub password: String,
    pub sample_type: Option<String>,  // "music", "it", "none" 或空
}

pub async fn install_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallRequest>,
) -> impl IntoResponse {


   // 1. 检查是否已安装
    let installed = sqlx::query("SELECT 1 FROM install_lock LIMIT 1")
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if installed {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Already installed"}))).into_response();
    }

    // 2. 创建管理员用户
    let hashed = match hash(&payload.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Hash error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Password hashing failed"}))).into_response();
        }
    };

    let admin_id = Uuid::new_v4();
    if let Err(e) = sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())"
    )
    .bind(admin_id)
    .bind(&payload.email)
    .bind(&hashed)
    .bind("管理员")
    .execute(&state.db_pool)
    .await
    {
        eprintln!("Create admin error: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create admin"}))).into_response();
    }

    // 3. 分配 admin 角色
    let admin_role_id: Option<i32> = sqlx::query_scalar("SELECT id FROM roles WHERE name = 'admin'")
        .fetch_optional(&state.db_pool)
        .await
        .unwrap_or(None);

    if let Some(role_id) = admin_role_id {
        if let Err(e) = sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(admin_id)
        .bind(role_id)
        .execute(&state.db_pool)
        .await
        {
            eprintln!("Assign admin role error: {}", e);
        }
    } else {
        eprintln!("Admin role not found! Make sure roles table is populated.");
    }

    // 4. 安装示例数据（如果请求中包含 sample_data = true）
    // 安装示例数据
   


    if let Some(ref sample_type) = payload.sample_type {
         if sample_type != "none" {
            match sample_type.as_str() {
                "music" => {
                    // 硬编码安装音乐示例数据（分类 + 文章）
                    if let Err(e) = install_music_sample(&state.db_pool).await {
                        let msg = format!("音乐示例数据安装失败: {}", e);
                        eprintln!("{}", msg);
                    }
                }
                "it" => {
                    // 企业示例数据（文件方式）
                    if let Err(e) = install_sample_data_from_file(&state.db_pool, "config/sample_data_it.sql").await {
                        let msg = format!("企业示例数据安装失败: {}", e);
                        eprintln!("{}", msg);
                    }
                }
                _ => {}
            }
        }
    }

    // 5. 写入安装锁
    if let Err(e) = sqlx::query("INSERT INTO install_lock (id) VALUES (TRUE)")
        .execute(&state.db_pool)
        .await
    {
        eprintln!("Lock error: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to lock installation"}))).into_response();
    }


    (StatusCode::OK, Json(json!({"message": "Installation successful"}))).into_response()
}

/// 从文件读取并执行 SQL
async fn install_sample_data_from_file(pool: &sqlx::PgPool, file_path: &str) -> Result<(), anyhow::Error> {
    let path = PathBuf::from(file_path);
    let sql_content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("读取示例数据文件失败 ({}): {}", file_path, e))?;

    // 按分号分割（简单分割，注意字符串内可能含分号，但本例中可控）
    let statements: Vec<&str> = sql_content
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut tx = pool.begin().await?;
    for stmt in statements {
        sqlx::query(stmt)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("执行 SQL 失败: {}\nSQL: {}", e, stmt))?;
    }
    tx.commit().await?;
    Ok(())
}

/// 硬编码安装音乐示例数据（分类 + 文章），使用 WHERE NOT EXISTS 避免依赖唯一约束
async fn install_music_sample(pool: &sqlx::PgPool) -> Result<(), anyhow::Error> {
    // ---- 插入分类 ----
    // 顶级分类
    sqlx::query(
        r#"
        INSERT INTO categories (name, slug, description, parent_id)
        SELECT '乐谱', 'score', '乐谱相关分类', NULL
        WHERE NOT EXISTS (SELECT 1 FROM categories WHERE slug = 'score');
        "#
    )
    .execute(pool)
    .await?;

    // 子分类
    sqlx::query(
        r#"
        INSERT INTO categories (name, slug, description, parent_id)
        SELECT '古典音乐', 'classical', '西方古典音乐作品与理论', (SELECT id FROM categories WHERE slug = 'score')
        WHERE NOT EXISTS (SELECT 1 FROM categories WHERE slug = 'classical');
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO categories (name, slug, description, parent_id)
        SELECT '流行音乐', 'pop', '现代流行、摇滚、电子等', (SELECT id FROM categories WHERE slug = 'score')
        WHERE NOT EXISTS (SELECT 1 FROM categories WHERE slug = 'pop');
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO categories (name, slug, description, parent_id)
        SELECT '爵士音乐', 'jazz', '爵士乐风格与即兴', (SELECT id FROM categories WHERE slug = 'score')
        WHERE NOT EXISTS (SELECT 1 FROM categories WHERE slug = 'jazz');
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO categories (name, slug, description, parent_id)
        SELECT '民族音乐', 'folk', '世界民族音乐、中国传统音乐', (SELECT id FROM categories WHERE slug = 'score')
        WHERE NOT EXISTS (SELECT 1 FROM categories WHERE slug = 'folk');
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO categories (name, slug, description, parent_id)
        SELECT '乐理与视唱', 'music-theory', '音阶、调式、节奏、视唱练耳', (SELECT id FROM categories WHERE slug = 'score')
        WHERE NOT EXISTS (SELECT 1 FROM categories WHERE slug = 'music-theory');
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO categories (name, slug, description, parent_id)
        SELECT '和声与作曲', 'harmony-composition', '和声学、曲式分析、作曲技术', (SELECT id FROM categories WHERE slug = 'score')
        WHERE NOT EXISTS (SELECT 1 FROM categories WHERE slug = 'harmony-composition');
        "#
    )
    .execute(pool)
    .await?;

    // ---- 插入文章 ----
    // 1. Cooley's
    sqlx::query(
        r#"
        INSERT INTO contents (id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at)
        SELECT
            uuid_generate_v4(),
            'cooleys-reel',
            'Cooley''s',
            '```abc
X: 1
T: Cooley''s
M: 4/4
L: 1/8
K: Emin
|:D2|"Em"EBBA B2 EB|\
    ~B2 AB dBAG|\
    "D"FDAD BDAD|\
    FDAD dAFD|
"Em"EBBA B2 EB|\
    B2 AB defg|\
    "D"afe^c dBAF|\
    "Em"DEFD E2:|
|:gf|"Em"eB B2 efge|\
    eB B2 gedB|\
    "D"A2 FA DAFA|\
    A2 FA defg|
"Em"eB B2 eBgB|\
    eB B2 defg|\
    "D"afe^c dBAF|\
    "Em"DEFD E2:|',
            true,
            NULL,
            'zh',
            uuid_generate_v4(),
            NOW(),
            NOW()
        WHERE NOT EXISTS (SELECT 1 FROM contents WHERE slug = 'cooleys-reel');
        "#
    )
    .execute(pool)
    .await?;

    // 关联到五线谱
    sqlx::query(
        r#"
        INSERT INTO content_categories (content_id, category_id)
        SELECT c.id, cat.id
        FROM contents c, categories cat
        WHERE c.slug = 'cooleys-reel' AND cat.slug = 'classical'
        AND NOT EXISTS (SELECT 1 FROM content_categories WHERE content_id = c.id AND category_id = cat.id);
        "#
    )
    .execute(pool)
    .await?;

    // 2. Clouds Thicken
    sqlx::query(
        r#"
        INSERT INTO contents (id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at)
        SELECT
            uuid_generate_v4(),
            'clouds-thicken',
            'Clouds Thicken',
            '```abc
X: 24
T: Clouds Thicken
C: Paul Rosen
S: Copyright 2005, Paul Rosen
M: 6/8
L: 1/8
Q: 3/8=116
R: Creepy Jig
K: Em
|:"Em"EEE E2G|"C7"_B2A G2F|"Em"EEE E2G|\
"C7"_B2A "B7"=B3|"Em"EEE E2G|
"C7"_B2A G2F|"Em"GFE "D (Bm7)"F2D|\
1"Em"E3-E3:|2"Em"E3-E2B|:"Em"e2e gfe|
"G"g2ab3|"Em"gfeg2e|"D"fedB2A|"Em"e2e gfe|\
"G"g2ab3|"Em"gfe"D"f2d|"Em"e3-e3:|',
            true,
            NULL,
            'zh',
            uuid_generate_v4(),
            NOW(),
            NOW()
        WHERE NOT EXISTS (SELECT 1 FROM contents WHERE slug = 'clouds-thicken');
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO content_categories (content_id, category_id)
        SELECT c.id, cat.id
        FROM contents c, categories cat
        WHERE c.slug = 'clouds-thicken' AND cat.slug = 'classical'
        AND NOT EXISTS (SELECT 1 FROM content_categories WHERE content_id = c.id AND category_id = cat.id);
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}
