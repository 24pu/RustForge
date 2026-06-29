use axum::{extract::State, Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::presentation::middleware::CurrentUser;
use crate::infrastructure::auth::generate_token;
use crate::core::UserRepository;
use uuid::Uuid;
use sqlx;
use tower_cookies::{Cookie, Cookies};
use serde::Deserialize;

pub async fn logout_handler(cookies: Cookies) -> impl IntoResponse {
    let removal = Cookie::build(("auth_token", ""))
        .path("/")
        .http_only(true)
        .same_site(tower_cookies::cookie::SameSite::Lax)
        .max_age(tower_cookies::cookie::time::Duration::seconds(0))
        .build();
    cookies.add(removal);
    Json(json!({ "message": "Logged out" }))
}

pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    if let Ok(Some(_)) = state.user_repo.get_user_by_email(&payload.email).await {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Email already exists" }))).into_response();
    }
    let hashed = match hash(&payload.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to hash password" }))).into_response(),
    };
    match state.user_repo.create_user(&payload.email, &hashed, payload.name.as_deref()).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "message": "User registered" }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create user" }))).into_response(),
    }
}

pub async fn login_handler(
    cookies: Cookies, 
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match state.user_repo.get_user_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid credentials" }))).into_response(),
    };
    let valid = match verify(&payload.password, &user.password_hash) {
        Ok(v) => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Error verifying password" }))).into_response(),
    };
    if !valid {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid credentials" }))).into_response();
    }
    let token = match generate_token(user.id) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to generate token" }))).into_response(),
    };

    cookies.add(
        Cookie::build(("auth_token", token.clone()))
            .path("/")
            .http_only(true)
            .same_site(tower_cookies::cookie::SameSite::Lax)
            .build(),
    );

    (StatusCode::OK, Json(json!({
        "success": true,
        "message": "Login successful",
        "token": token
    }))).into_response()
}

pub async fn me_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response(),
    };
    let user = match state.user_repo.get_user_by_id(user_id).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "error": "User not found" }))).into_response(),
    };
    let roles = state.user_repo.get_user_roles(user_id).await.unwrap_or_default();
    (StatusCode::OK, Json(json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "roles": roles,
    }))).into_response()
}

pub async fn my_permissions_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    let rows = match sqlx::query!(
        "SELECT DISTINCT p.name FROM user_roles ur
         JOIN role_permissions rp ON ur.role_id = rp.role_id
         JOIN permissions p ON rp.permission_id = p.id
         WHERE ur.user_id = $1",
        user_id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch permissions: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch permissions").into_response();
        }
    };
    let permissions: Vec<String> = rows.into_iter().map(|r| r.name).collect();
    (StatusCode::OK, Json(permissions)).into_response()
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => {
            //eprintln!("[change_password] Unauthorized: user_id is None");
            return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response();
        }
    };
   // eprintln!("[change_password] User ID: {}", user_id);

    let user = match state.user_repo.get_user_by_id(user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            //eprintln!("[change_password] User not found");
            return (StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))).into_response();
        }
        Err(e) => {
            //eprintln!("[change_password] Database error: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database error"}))).into_response();
        }
    };

    //eprintln!("[change_password] password_hash from DB: {}", user.password_hash);

    match verify(&payload.old_password, &user.password_hash) {
        Ok(true) => {
            //eprintln!("[change_password] Old password matched");
        }
        Ok(false) => {
           // eprintln!("[change_password] Old password mismatch");
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "当前密码错误"}))).into_response();
        }
        Err(e) => {
           // eprintln!("[change_password] bcrypt verify error: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "密码验证出错"}))).into_response();
        }
    }

    let new_hash = match hash(&payload.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            //eprintln!("[change_password] bcrypt hash error: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "密码加密失败"}))).into_response();
        }
    };

    match state.user_repo.update_password(user_id, &new_hash).await {
        Ok(()) => {
            //eprintln!("[change_password] Password updated successfully");
            (StatusCode::OK, Json(json!({"message": "密码修改成功"}))).into_response()
        }
        Err(e) => {
            //eprintln!("[change_password] Update error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "更新密码失败"}))).into_response()
        }
    }
}