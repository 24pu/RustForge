use axum::{
    extract::{State, Path, Multipart},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum::extract::FromRequest;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use std::path::Path as StdPath;
use uuid::Uuid;
use crate::presentation::AppState;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::check_permission;
use crate::core::models::Plugin;
use crate::infrastructure::plugin::WasmPlugin;
use crate::presentation::types::InstallPluginRequest;

#[derive(Serialize)]
pub struct AvailablePlugin {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub file_path: String,
}

pub async fn scan_available_plugins_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let plugins_dir = StdPath::new("plugins");
    let mut available = Vec::new();

    if let Ok(entries) = std::fs::read_dir(plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(wasm_files) = std::fs::read_dir(&path) {
                    for wasm_entry in wasm_files.flatten() {
                        let wasm_path = wasm_entry.path();
                        if wasm_path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                            if let Ok(mut wasm) = WasmPlugin::load(wasm_path.to_str().unwrap()) {
                                if let Ok(metadata_str) = wasm.call_metadata() {
                                    if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&metadata_str) {
                                        let name = meta["name"].as_str().unwrap_or("unknown").to_string();
                                        if let Ok(Some(_)) = state.plugin_repo.get_plugin_by_name(&name).await {
                                            continue; // 已安装，跳过
                                        }
                                        available.push(AvailablePlugin {
                                            name,
                                            version: meta["version"].as_str().unwrap_or("0.0.0").to_string(),
                                            author: meta["author"].as_str().unwrap_or("").to_string(),
                                            description: meta["description"].as_str().unwrap_or("").to_string(),
                                            file_path: wasm_path.to_string_lossy().to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (StatusCode::OK, Json(available))
}

pub async fn install_plugin_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Response {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "plugin:install").await {
        return (status, msg).into_response();
    }

    let content_type = req.headers().get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let response = if content_type.contains("application/json") {
        let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
            Ok(b) => b,
            Err(e) => return (StatusCode::BAD_REQUEST, format!("Failed to read body: {}", e)).into_response(),
        };
        let payload: InstallPluginRequest = match serde_json::from_slice(&body_bytes) {
            Ok(p) => p,
            Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)).into_response(),
        };
        if let Some(file_path) = payload.file_path {
            install_plugin_by_path(&state, file_path).await
        } else {
            (StatusCode::BAD_REQUEST, "Missing file_path").into_response()
        }
    } else {
        let multipart = match Multipart::from_request(req, &()).await {
            Ok(m) => m,
            Err(e) => return (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)).into_response(),
        };
        handle_multipart_install(&state, multipart).await
    };

    // 安装成功后重新加载语言包
    if matches!(response.status(), StatusCode::CREATED) {
        let supported_langs = state.i18n.supported_langs();
        let default_lang = state.i18n.default_lang();
        state.i18n.reload_with_plugins(supported_langs, default_lang, &state.db_pool).await;
    }

    response
}

async fn install_plugin_by_path(state: &Arc<AppState>, file_path: String) -> Response {
    let source = StdPath::new(&file_path);
    if !source.exists() {
        return (StatusCode::BAD_REQUEST, "Plugin file not found").into_response();
    }

    let mut wasm = match WasmPlugin::load(&file_path) {
        Ok(w) => w,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid plugin: {}", e)).into_response(),
    };
    let metadata_str = match wasm.call_metadata() {
        Ok(s) => s,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Metadata error: {}", e)).into_response(),
    };
    let metadata: serde_json::Value = match serde_json::from_str(&metadata_str) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid metadata JSON: {}", e)).into_response(),
    };
    let name = metadata.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let version = metadata.get("version").and_then(|v| v.as_str()).unwrap_or("1.0").to_string();
    let author = metadata.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
    let description = metadata.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

    if let Ok(Some(_)) = state.plugin_repo.get_plugin_by_name(&name).await {
        return (StatusCode::CONFLICT, format!("Plugin '{}' already installed", name)).into_response();
    }

    let safe_name = name.replace(' ', "_").to_lowercase();
    let unique_name = format!("{}_{}.wasm", safe_name, Uuid::new_v4());
    let dest_path = format!("plugins/{}", unique_name);
    let dest_full_path = StdPath::new(&dest_path).to_path_buf(); // 转换为 owned PathBuf 避免借用冲突
    if let Some(parent) = dest_full_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create directory: {}", e)).into_response();
        }
    }
    if let Err(e) = tokio::fs::copy(source, &dest_full_path).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to copy plugin file: {}", e)).into_response();
    }

    let plugin = Plugin {
        id: 0,
        name,
        version,
        author,
        description,
        file_path: dest_path,   // 可以安全移动
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match state.plugin_repo.create_plugin(&plugin).await {
        Ok(p) => (StatusCode::CREATED, Json(p)).into_response(),
        Err(e) => {
            let _ = tokio::fs::remove_file(&dest_full_path).await;
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

async fn handle_multipart_install(state: &Arc<AppState>, mut multipart: Multipart) -> Response {
    let mut file_data = None;
    let mut file_name = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            file_name = field.file_name().map(|s| s.to_string());
            if let Ok(data) = field.bytes().await { file_data = Some(data); }
            break;
        }
    }

    let (file_data, file_name) = match (file_data, file_name) {
        (Some(d), Some(n)) => (d, n),
        _ => return (StatusCode::BAD_REQUEST, "Missing plugin file").into_response(),
    };

    let temp_dir = std::env::temp_dir();
    let temp_name = format!("upload_{}.wasm", Uuid::new_v4());
    let temp_path = temp_dir.join(&temp_name);
    if let Err(e) = tokio::fs::write(&temp_path, &file_data).await {
        eprintln!("Failed to write temp file: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save plugin").into_response();
    }

    let mut wasm_plugin = match WasmPlugin::load(temp_path.to_str().unwrap()) {
        Ok(p) => p,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return (StatusCode::BAD_REQUEST, format!("Invalid plugin: {}", e)).into_response();
        }
    };
    let metadata_str = match wasm_plugin.call_metadata() {
        Ok(s) => s,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return (StatusCode::BAD_REQUEST, format!("Plugin missing metadata: {}", e)).into_response();
        }
    };
    let metadata: serde_json::Value = match serde_json::from_str(&metadata_str) {
        Ok(v) => v,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return (StatusCode::BAD_REQUEST, format!("Invalid metadata JSON: {}", e)).into_response();
        }
    };
    let name = metadata.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let version = metadata.get("version").and_then(|v| v.as_str()).unwrap_or("1.0").to_string();
    let author = metadata.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
    let description = metadata.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

    let safe_name = name.replace(' ', "_").to_lowercase();
    let unique_name = format!("{}_{}.wasm", safe_name, Uuid::new_v4());
    let final_path = format!("plugins/{}", unique_name);
    let final_full_path = StdPath::new(&final_path).to_path_buf(); // 转为 PathBuf
    if let Some(parent) = final_full_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Err(e) = tokio::fs::rename(&temp_path, &final_full_path).await {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to move plugin file").into_response();
    }

    if let Ok(Some(_)) = state.plugin_repo.get_plugin_by_name(&name).await {
        let _ = tokio::fs::remove_file(&final_full_path).await;
        return (StatusCode::CONFLICT, format!("Plugin '{}' already installed", name)).into_response();
    }

    let plugin = Plugin {
        id: 0,
        name,
        version,
        author,
        description,
        file_path: final_path,   // 安全移动
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    match state.plugin_repo.create_plugin(&plugin).await {
        Ok(p) => (StatusCode::CREATED, Json(p)).into_response(),
        Err(e) => {
            let _ = tokio::fs::remove_file(&final_full_path).await;
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn toggle_plugin_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "plugin:manage").await {
        return (status, msg).into_response();
    }
    let enabled = match payload.get("enabled").and_then(|v| v.as_bool()) {
        Some(e) => e,
        None => return (StatusCode::BAD_REQUEST, "Missing enabled field").into_response(),
    };
    match state.plugin_repo.update_plugin(id, enabled).await {
        Ok(()) => {
            // 重新加载语言包（包含已启用插件的翻译）
            let supported_langs = state.i18n.supported_langs();
            let default_lang = state.i18n.default_lang();
            state.i18n.reload_with_plugins(supported_langs, default_lang, &state.db_pool).await;
            
            (StatusCode::OK, Json(json!({ "message": "Plugin updated" }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn uninstall_plugin_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "plugin:uninstall").await {
        return (status, msg).into_response();
    }

    let plugin = match state.plugin_repo.get_plugin_by_id(id).await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "Plugin not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    match state.plugin_repo.delete_plugin(id).await {
        Ok(true) => {}
        Ok(false) => return (StatusCode::NOT_FOUND, "Plugin not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }

    // 删除物理文件
    let _ = tokio::fs::remove_file(&plugin.file_path).await;

    // 重新加载语言包（移除已卸载插件的翻译）
    let supported_langs = state.i18n.supported_langs();
    let default_lang = state.i18n.default_lang();
    state.i18n.reload_with_plugins(supported_langs, default_lang, &state.db_pool).await;

    (StatusCode::NO_CONTENT, "").into_response()
}

pub async fn list_plugins_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "plugin:list").await {
        return (status, msg).into_response();
    }
    match state.plugin_repo.list_plugins().await {
        Ok(plugins) => (StatusCode::OK, Json(plugins)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}