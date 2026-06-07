use axum::{extract::State, Json, http::StatusCode, response::IntoResponse};
use serde_json::{json, Value, Map};
use std::sync::Arc;
use sqlx::Row;

use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::presentation::handlers::utils::check_permission;
use crate::presentation::middleware::CurrentUser;

use uuid::Uuid;


pub async fn get_config_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let rows = match sqlx::query!("SELECT key, value FROM site_config")
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch config: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to load config" }))).into_response();
        }
    };
    let mut config = Map::new();
    for row in rows {
        config.insert(row.key, Value::String(row.value));
    }
    let default_fields = [
        "seo_title", "seo_description", "seo_keywords",
        "logo_url", "favicon_url",
        "allowed_file_types", "max_file_size_mb"
    ];
    for field in default_fields {
        if !config.contains_key(field) {
            config.insert(field.to_string(), Value::String("".to_string()));
        }
    }
    (StatusCode::OK, Json(config)).into_response()
}

pub async fn update_config_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "config:edit").await {
        return (status, msg).into_response();
    }

    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to begin transaction: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to start transaction" }))).into_response();
        }
    };

    let config_items = vec![
        ("site_name", payload.site_name),
        ("default_per_page", payload.default_per_page.to_string()),
        ("theme_color", payload.theme_color),
        ("seo_title", payload.seo_title),
        ("seo_description", payload.seo_description),
        ("seo_keywords", payload.seo_keywords),
        ("logo_url", payload.logo_url),
        ("favicon_url", payload.favicon_url),
        ("allowed_file_types", payload.allowed_file_types),
        ("max_file_size_mb", payload.max_file_size_mb.to_string()),
    ];

    for (key, value) in config_items {
        if let Err(e) = sqlx::query!(
            "INSERT INTO site_config (key, value) VALUES ($1, $2)
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
            key,
            value
        )
        .execute(&mut *tx)
        .await
        {
            eprintln!("Failed to update config key '{}': {}", key, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to update {}", key) }))).into_response();
        }
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Failed to commit transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to save config" }))).into_response();
    }

    (StatusCode::OK, Json(json!({ "message": "Config updated" }))).into_response()
}