use axum::{extract::{State, Path}, Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::check_permission;
use crate::core::UserRepository;

use uuid::Uuid;


pub async fn list_roles_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "role:list").await {
        return (status, msg).into_response();
    }
    match state.user_repo.list_roles().await {
        Ok(roles) => (StatusCode::OK, Json(roles)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch roles").into_response(),
    }
}

pub async fn create_role_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRoleRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "role:create").await {
        return (status, msg).into_response();
    }
    match state.user_repo.create_role(&payload.name, payload.description.as_deref()).await {
        Ok(role) => (StatusCode::CREATED, Json(role)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create role: {}", e)).into_response(),
    }
}

pub async fn update_role_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
    Json(payload): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "role:edit").await {
        return (status, msg).into_response();
    }
    match state.user_repo.update_role(role_id, &payload.name, payload.description.as_deref()).await {
        Ok(role) => (StatusCode::OK, Json(role)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update role: {}", e)).into_response(),
    }
}

pub async fn delete_role_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "role:delete").await {
        return (status, msg).into_response();
    }
    match state.user_repo.delete_role(role_id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Role not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete role").into_response(),
    }
}

pub async fn list_permissions_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "role:list").await {
        return (status, msg).into_response();
    }
    match state.user_repo.list_permissions().await {
        Ok(perms) => (StatusCode::OK, Json(perms)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch permissions").into_response(),
    }
}

pub async fn get_role_permissions_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "role:list").await {
        return (status, msg).into_response();
    }
    match state.user_repo.get_role_permissions(role_id).await {
        Ok(perms) => (StatusCode::OK, Json(perms)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch role permissions").into_response(),
    }
}

pub async fn update_role_permissions_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
    Json(payload): Json<UpdateRolePermissionsRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response(),
    };
    if !state.user_repo.user_has_permission(user_id, "role:assign").await.unwrap_or(false) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response();
    }
    match state.user_repo.update_role_permissions(role_id, &payload.permission_ids).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "message": "Permissions updated" }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to update permissions" }))).into_response(),
    }
}