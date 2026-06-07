
use axum::{
    extract::{Path, State, Query, Extension},   // 添加 Extension
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use crate::presentation::types::UserInfo;        // 添加 UserInfo
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::presentation::AppState;
use crate::presentation::middleware::CurrentUser;

use crate::infrastructure::plugin::WasmPlugin;
use crate::presentation::handlers::utils::{get_nav_categories, get_site_config_map};
use crate::core::ThemeManager;
use tower_cookies::Cookies;
use crate::infrastructure::auth::verify_token;
use axum::extract::Request;
use crate::infrastructure::i18n::LangOption;

#[derive(serde::Deserialize)]
pub struct PluginCallParams {
    pub input: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}



pub async fn plugin_gateway_handler(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Path(plugin_name): Path<String>,
    query: Option<Query<PluginCallParams>>,
    body: Option<Json<Value>>,
) -> impl IntoResponse {
    // 1. 提取当前用户 ID
    let token = cookies.get("auth_token").map(|c| c.value().to_string());
    let user_id = token.and_then(|t| verify_token(&t).ok()).map(|claims| claims.sub);

    // 2. 构建原始 input JSON（从 body 或 query）
    let mut input_json = if let Some(Json(body_val)) = body {
        body_val
    } else if let Some(Query(params)) = query {
        if let Some(input_str) = &params.input {
            serde_json::from_str(input_str).unwrap_or(Value::String(input_str.clone()))
        } else {
            serde_json::to_value(&params.extra).unwrap_or(Value::Null)
        }
    } else {
        Value::Null
    };

    // 3. 注入 user_id
    if let Some(uid) = user_id {
        if let Value::Object(ref mut map) = input_json {
            map.insert("user_id".to_string(), Value::String(uid));
        } else {
            return (StatusCode::BAD_REQUEST, "Plugin input must be a JSON object").into_response();
        }
    }

    // 4. 检查插件可用性
    let plugin = match state.plugin_repo.get_plugin_by_name(&plugin_name).await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Plugin '{}' not found", plugin_name)).into_response(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };
    if !plugin.enabled {
        return (StatusCode::FORBIDDEN, format!("Plugin '{}' is disabled", plugin_name)).into_response();
    }

    // 5. 调用 Wasm 插件（使用已注入 user_id 的 input_json）
    let input_string = input_json.to_string();   // 现在只用一次
    let mut wasm_plugin = match WasmPlugin::load(&plugin.file_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load plugin {}: {}", plugin_name, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load plugin: {}", e)).into_response();
        }
    };
    match wasm_plugin.call_execute(&input_string) {
        Ok(output) => {
            match serde_json::from_str::<Value>(&output) {
                Ok(json_val) => Json(json_val).into_response(),
                Err(_) => output.into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Plugin execution error: {}", e)).into_response(),
    }
}
// 静态文件处理器
pub async fn plugin_view_handler(
    Path((plugin_name, page)): Path<(String, String)>,
) -> impl IntoResponse {
    let view_path = format!("plugins/{}/views/{}.html", plugin_name, page);
    match tokio::fs::read_to_string(view_path).await {
        Ok(content) => ([(header::CONTENT_TYPE, "text/html")], content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

pub async fn plugin_static_handler(
    Path((plugin_name, file_path)): Path<(String, String)>,
) -> impl IntoResponse {
    let full_path = format!("plugins/{}/static/{}", plugin_name, file_path);
    match tokio::fs::read(&full_path).await {
        Ok(data) => {
            let content_type = mime_guess::from_path(&full_path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, content_type.as_ref())], data).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}





pub async fn plugin_page_handler(
    Extension(user_info): Extension<UserInfo>,
    Path((plugin_name, page)): Path<(String, String)>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = user_info.user_id.clone();

    let plugin = match state.plugin_repo.get_plugin_by_name(&plugin_name).await {
        Ok(Some(p)) if p.enabled => p,
        _ => return (StatusCode::NOT_FOUND, "Plugin not found or disabled").into_response(),
    };

    let is_protected = if let Ok(mut wasm) = WasmPlugin::load(&plugin.file_path) {
        match wasm.call_is_page_protected(&page).await {
            Ok(result) => result.trim().to_lowercase() == "true",
            Err(_) => false,
        }
    } else {
        matches!(page.as_str(), "profile" | "orders" | "password")
    };

    if is_protected && user_id.is_none() {
        return Redirect::temporary(&format!("/plugins/{}/login", plugin_name)).into_response();
    }

    let plugin_locales = load_plugin_locales(&plugin_name, &lang);

    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;
    let view_path = format!("plugins/{}/views/{}.html", plugin_name, page);
    let mut content_fragment = match tokio::fs::read_to_string(&view_path).await {
        Ok(c) => c,
        Err(_) => return (StatusCode::NOT_FOUND, "Page not found").into_response(),
    };

    // 直接替换模板中的变量
    for (key, value) in &plugin_locales {
        let placeholder = format!("{{{{ {} }}}}", key);
        content_fragment = content_fragment.replace(&placeholder, value);
    }

    let final_content = format!(
        r#"<script>window.__plugin_locales__ = {};</script>
        <script src="/admin/common.js"></script>
        {}"#,
        serde_json::to_string(&plugin_locales).unwrap_or_else(|_| "{}".to_string()),
        content_fragment
    );

    let mut context = HashMap::new();
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("content".to_string(), json!(final_content));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
    }));
    context.insert("lang".to_string(), json!(lang));
    context.insert("lang_options".to_string(), json!(lang_options));
    context.insert("plugin_locales".to_string(), json!(plugin_locales));

    let theme_manager = state.theme_manager.read().await;
    match theme_manager.render("base.html", context).await {
        Ok(html) => ([(header::CONTENT_TYPE, "text/html")], html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn load_plugin_locales(plugin_name: &str, lang: &str) -> HashMap<String, String> {
    let locales_path = format!("plugins/{}/locales/{}.json", plugin_name, lang);
    match std::fs::read_to_string(&locales_path) {
        Ok(content) => {
            match serde_json::from_str::<HashMap<String, String>>(&content) {
                Ok(map) => return map,
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
    HashMap::new()
}