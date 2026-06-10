use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::sync::Arc;
use std::fs;
use tokio::fs as tokio_fs;
use crate::presentation::AppState;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::check_permission;
use crate::core::ThemeManager;
use std::path::{Path as StdPath};

#[derive(Serialize)]
pub struct ThemeInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct ActivateThemeRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct TemplateFile {
    pub name: String,
    pub path: String,
}

// 获取所有主题列表
pub async fn list_themes_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "theme:list").await {
        return (status, msg).into_response();
    }

    let themes_dir = StdPath::new(&state.config.theme.themes_dir);
    let mut themes = Vec::new();

    // 从主题管理器获取当前激活的主题名称
    let active_theme = {
        let manager = state.theme_manager.read().await;
        manager.active_theme()
    };

    if let Ok(entries) = fs::read_dir(themes_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let meta_path = path.join("theme.toml");
                if meta_path.exists() {
                    if let Ok(content) = fs::read_to_string(&meta_path) {
                        if let Ok(meta) = toml::from_str::<ThemeMetadata>(&content) {
                            let is_active = active_theme == meta.name;
                            themes.push(ThemeInfo {
                                name: meta.name,
                                version: meta.version,
                                author: meta.author,
                                description: meta.description,
                                is_active,
                            });
                        }
                    }
                }
            }
        }
    }

    (StatusCode::OK, Json(themes)).into_response()
}

// 切换当前主题（更新 config.toml 并热切换）
pub async fn activate_theme_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ActivateThemeRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "theme:activate").await {
        return (status, msg).into_response();
    }

    let theme_dir = StdPath::new(&state.config.theme.themes_dir).join(&payload.name);
    if !theme_dir.exists() || !theme_dir.is_dir() {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Theme not found" }))).into_response();
    }

    // 更新 config.toml
    let config_path = StdPath::new("config.toml");
    let config_content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read config.toml: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to read config" }))).into_response();
        }
    };
    let mut doc: toml::Value = match toml::from_str(&config_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to parse config.toml: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Invalid config format" }))).into_response();
        }
    };
    doc["theme"]["default_theme"] = toml::Value::String(payload.name.clone());
    let new_content = match toml::to_string(&doc) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize config: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to save config" }))).into_response();
        }
    };
    if let Err(e) = fs::write(config_path, new_content) {
        eprintln!("Failed to write config.toml: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to write config" }))).into_response();
    }

    // 立即热切换主题
    {
        let mut manager = state.theme_manager.write().await;
        if let Err(e) = manager.set_active_theme(&payload.name) {
            eprintln!("Failed to set active theme: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to activate theme: {:?}", e) }))).into_response();
        }
    }

    (StatusCode::OK, Json(json!({ "message": "Theme activated successfully" }))).into_response()
}

// 获取指定主题的模板文件列表
pub async fn list_templates_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(theme_name): Path<String>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "theme:edit").await {
        return (status, msg).into_response();
    }

    let theme_dir = StdPath::new(&state.config.theme.themes_dir).join(&theme_name);
    let templates_dir = theme_dir.join("templates");
    if !templates_dir.exists() {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Templates directory not found" }))).into_response();
    }

    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&templates_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("html") {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let relative_path = format!("templates/{}", name);
                files.push(TemplateFile { name, path: relative_path });
            }
        }
    }

    (StatusCode::OK, Json(files)).into_response()
}

// 获取模板文件内容
pub async fn get_template_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path((theme_name, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "theme:edit").await {
        return (status, msg).into_response();
    }

    let file_path = StdPath::new(&state.config.theme.themes_dir)
        .join(&theme_name)
        .join("templates")
        .join(&filename);

    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Template not found" }))).into_response();
    }

    match tokio_fs::read_to_string(&file_path).await {
        Ok(content) => (StatusCode::OK, Json(json!({ "content": content }))).into_response(),
        Err(e) => {
            eprintln!("Failed to read template: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to read file" }))).into_response()
        }
    }
}

pub async fn update_template_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path((theme_name, filename)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "theme:edit").await {
        return (status, msg).into_response();
    }

    let content = match payload.get("content").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Missing content field" }))).into_response(),
    };

    let file_path = StdPath::new(&state.config.theme.themes_dir)
        .join(&theme_name)
        .join("templates")
        .join(&filename);

    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Template not found" }))).into_response();
    }

    if let Err(e) = tokio_fs::write(&file_path, content).await {
        eprintln!("Failed to write template: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to write file" }))).into_response();
    }

    // 重载该主题的模板
    let manager = state.theme_manager.write().await;
    match manager.reload_theme(&theme_name).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "message": "Template updated and reloaded" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Template saved but reload failed: {}", e) }))).into_response(),
    }
}

// 辅助结构体
#[derive(Deserialize)]
struct ThemeMetadata {
    name: String,
    version: String,
    author: String,
    description: Option<String>,
}