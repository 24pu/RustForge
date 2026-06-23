use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::core::AmaTemplateRepository;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::check_permission;

pub async fn list_amatemplates_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "template:list").await {
        return (status, Json(json!({"error": msg}))).into_response();
    }
    match state.amatemplate_repo.list(user_id).await {
        Ok(templates) => (StatusCode::OK, Json(templates)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn create_amatemplate_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAmaTemplateRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "template:create").await {
        return (status, Json(json!({"error": msg}))).into_response();
    }
    let is_used = payload.is_used.unwrap_or(false);
    match state.amatemplate_repo.create(&payload.name, &payload.value, is_used, user_id).await {
        Ok(tmpl) => (StatusCode::CREATED, Json(tmpl)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_amatemplate_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };
    
    match state.amatemplate_repo.get_by_id(id, user_id).await {
        Ok(Some(tmpl)) => (StatusCode::OK, Json(tmpl)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn update_amatemplate_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateAmaTemplateRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "template:edit").await {
        return (status, Json(json!({"error": msg}))).into_response();
    }
    let existing = match state.amatemplate_repo.get_by_id(id, user_id).await {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };
    let name = payload.name.unwrap_or(existing.name);
    let value = payload.value.unwrap_or(existing.value);
    let is_used = payload.is_used.unwrap_or(existing.is_used);
    match state.amatemplate_repo.update(id, &name, &value, is_used, user_id).await {
        Ok(tmpl) => (StatusCode::OK, Json(tmpl)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn delete_amatemplate_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "template:delete").await {
        return (status, Json(json!({"error": msg}))).into_response();
    }
    match state.amatemplate_repo.delete(id, user_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}