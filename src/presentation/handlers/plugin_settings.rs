use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::presentation::AppState;


pub async fn get_plugin_settings(
    State(state): State<Arc<AppState>>,
    Path(plugin_name): Path<String>,
) -> impl IntoResponse {
    match state.plugin_settings_repo.get_settings(&plugin_name).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to load settings"})),
        ).into_response(),
    }
}

pub async fn update_plugin_settings(
    State(state): State<Arc<AppState>>,
    Path(plugin_name): Path<String>,
    Json(settings): Json<Value>,
) -> impl IntoResponse {
    match state.plugin_settings_repo.save_settings(&plugin_name, settings).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Settings saved"}))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to save settings"})),
        ).into_response(),
    }
}

pub async fn plugin_settings_view_handler(
    Path(plugin_name): Path<String>,
) -> impl IntoResponse {
    let view_path = format!("plugins/{}/views/settings.html", plugin_name);
    match tokio::fs::read_to_string(&view_path).await {
        Ok(content) => ([(header::CONTENT_TYPE, "text/html")], content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Settings page not found").into_response(),
    }
}

/// 公开接口：获取任意插件的设置（用于前端展示）
pub async fn get_public_plugin_settings(
    State(state): State<Arc<AppState>>,
    Path(plugin_name): Path<String>,
) -> impl IntoResponse {
    match state.plugin_settings_repo.get_settings(&plugin_name).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to load settings"})),
        ).into_response(),
    }
}