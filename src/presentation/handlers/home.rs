use axum::{
    extract::State,
    response::{Html, IntoResponse},
    http::StatusCode,
    Extension,                         // 新增
};
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;
use crate::presentation::AppState;
use crate::presentation::handlers::utils::get_nav_categories;
use crate::core::ThemeManager;
use crate::presentation::handlers::utils::get_site_config_map;
use crate::presentation::types::UserInfo;   // 确保导入 UserInfo
use crate::infrastructure::i18n::LangOption;

pub async fn home_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 获取最新内容，过滤当前语言（或显示所有语言，由模板过滤）
    let latest = state.content_repo.list_published(6).await.unwrap_or_default();
    // 过滤当前语言的内容
    let filtered_latest: Vec<_> = latest.into_iter()
        .filter(|c| c.lang == lang || c.lang.is_empty())
        .collect();
    
    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;

    let mut context = HashMap::new();
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("latest_contents".to_string(), json!(filtered_latest));
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
    }));
    context.insert("lang".to_string(), json!(lang));
    context.insert("lang_options".to_string(), json!(state.i18n.lang_options()));

    match state.theme_manager.read().await.render("index.html", context).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Render error: {}", e)).into_response(),
    }
}