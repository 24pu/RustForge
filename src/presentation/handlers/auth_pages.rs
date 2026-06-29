// src/presentation/handlers/auth_pages.rs

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::ThemeManager;
use crate::presentation::AppState;
use crate::presentation::handlers::utils::{get_nav_categories, get_site_config_map};
use crate::presentation::types::UserInfo;
use crate::infrastructure::i18n::LangOption;

/// 登录页
pub async fn login_page_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    // 如果已登录，跳转到用户中心
    if user_info.is_logged_in {
        return axum::response::Redirect::temporary("/user/profile").into_response();
    }

    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;

    let mut context = HashMap::new();
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("lang".to_string(), json!(lang));
    context.insert("lang_options".to_string(), json!(lang_options));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": false,
        "user_name": null,
    }));

    let theme_manager = state.theme_manager.read().await;
    match theme_manager.render("login.html", context).await {
        Ok(html) => ([(axum::http::header::CONTENT_TYPE, "text/html")], html).into_response(),
        Err(e) => {
            eprintln!("渲染登录页失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "页面渲染失败").into_response()
        }
    }
}

/// 注册页
pub async fn register_page_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if user_info.is_logged_in {
        return axum::response::Redirect::temporary("/user/profile").into_response();
    }

    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;

    let mut context = HashMap::new();
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("lang".to_string(), json!(lang));
    context.insert("lang_options".to_string(), json!(lang_options));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": false,
        "user_name": null,
    }));

    let theme_manager = state.theme_manager.read().await;
    match theme_manager.render("register.html", context).await {
        Ok(html) => ([(axum::http::header::CONTENT_TYPE, "text/html")], html).into_response(),
        Err(e) => {
            eprintln!("渲染注册页失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "页面渲染失败").into_response()
        }
    }
}

/// 修改密码页
pub async fn user_password_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if !user_info.is_logged_in {
        return axum::response::Redirect::temporary("/login").into_response();
    }

    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;

    let mut context = HashMap::new();
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("lang".to_string(), json!(lang));
    context.insert("lang_options".to_string(), json!(lang_options));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
        "user_id": user_info.user_id,
    }));
    context.insert("current_page".to_string(), json!("password"));

    let theme_manager = state.theme_manager.read().await;
    match theme_manager.render("user_password.html", context).await {
        Ok(html) => ([(axum::http::header::CONTENT_TYPE, "text/html")], html).into_response(),
        Err(e) => {
            eprintln!("渲染修改密码页失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "页面渲染失败").into_response()
        }
    }
}