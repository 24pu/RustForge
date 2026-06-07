use axum::{extract::{State, Path}, Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::check_permission;
use crate::core::UserRepository;

use uuid::Uuid;


pub async fn list_users_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response(),
    };
    let roles = state.user_repo.get_user_roles(user_id).await.unwrap_or_default();
    if !roles.contains(&"admin".to_string()) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response();
    }
    match state.user_repo.list_users_with_roles(100, 0).await {
        Ok(users) => {
            let users_json: Vec<_> = users.into_iter().map(|(user, roles)| {
                json!({
                    "id": user.id,
                    "email": user.email,
                    "name": user.name,
                    "roles": roles,
                    "created_at": user.created_at,
                })
            }).collect();
            (StatusCode::OK, Json(users_json)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch users" }))).into_response(),
    }
}

pub async fn update_user_roles_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(target_user_id): Path<Uuid>,
    Json(payload): Json<UpdateRolesRequest>,
) -> impl IntoResponse {
    let admin_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response(),
    };
    let has_perm = state.user_repo.user_has_permission(admin_id, "role:assign").await.unwrap_or(false);
    if !has_perm {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response();
    }
    let current_roles = state.user_repo.get_user_roles(target_user_id).await.unwrap_or_default();
    for role in current_roles {
        let _ = state.user_repo.revoke_role_by_name(target_user_id, &role).await;
    }
    for role_name in payload.roles {
        let _ = state.user_repo.assign_role_by_name(target_user_id, &role_name).await;
    }
    (StatusCode::OK, Json(json!({ "message": "Roles updated" }))).into_response()
}

pub async fn delete_user_handler(
    CurrentUser(current_user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let current_id = match current_user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    let has_perm = state.user_repo.user_has_permission(current_id, "user:delete").await.unwrap_or(false);
    if !has_perm {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    if current_id == user_id {
        return (StatusCode::BAD_REQUEST, "Cannot delete yourself").into_response();
    }
    match state.user_repo.delete_user(user_id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete user").into_response(),
    }
}