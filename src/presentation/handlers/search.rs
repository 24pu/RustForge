use axum::{extract::State, response::{Html, IntoResponse}, Extension};
use std::sync::Arc;
use std::collections::HashMap;
use crate::presentation::AppState;
use crate::presentation::types::UserInfo;
use crate::infrastructure::i18n::LangOption;
use crate::core::ThemeManager;   // 必须导入

pub async fn search_page_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = HashMap::new();
    let site_config = crate::presentation::handlers::utils::get_site_config_map(&state.db_pool).await;
    let nav_categories = crate::presentation::handlers::utils::get_nav_categories(&state).await;
    context.insert("site_config".to_string(), serde_json::json!(site_config));
    context.insert("nav_categories".to_string(), serde_json::json!(nav_categories));
    context.insert("user_info".to_string(), serde_json::json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
    }));
    context.insert("lang".to_string(), serde_json::json!(lang));
    context.insert("lang_options".to_string(), serde_json::json!(lang_options));
    match state.theme_manager.read().await.render("search.html", context).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("Render error: {}", e)).into_response(),
    }
}