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
use sqlx::migrate::Migrator;
use std::path::Path;

use crate::presentation::AppState;

#[derive(Deserialize)]
pub struct InstallRequest {
    pub email: String,
    pub password: String,
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

    // 2. 执行数据库迁移（如果之前已注释，保持注释；如果恢复，也OK）
    // 注意：由于表已手动创建，迁移可能会跳过或失败，建议保持注释
    /*
    let migrator = ...;
    migrator.run(&state.db_pool).await?;
    */

    // 3. 创建管理员用户
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

    // 4. 分配 admin 角色
    // 获取 admin 角色的 ID
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
            // 不中断安装，但提示错误
        }
    } else {
        eprintln!("Admin role not found! Make sure roles table is populated.");
        // 可以尝试手动插入角色，但理论上 roles 表已在迁移中初始化
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