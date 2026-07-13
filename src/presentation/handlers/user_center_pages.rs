// src/presentation/handlers/user_center_pages.rs

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

/// 用户中心通用页面渲染器
async fn render_user_page(
    state: &Arc<AppState>,
    user_info: &UserInfo,
    lang: &str,
    lang_options: &[LangOption],
    current_page: &str,
    extra_context: HashMap<String, serde_json::Value>,
) -> Response {
    let nav_categories = get_nav_categories(state).await;
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
    context.insert("current_page".to_string(), json!(current_page));

    // 合并额外上下文
    for (k, v) in extra_context {
        context.insert(k, v);
    }

    // 模板名称：user_center_base.html 作为基础，内容块由子模板填充
    // 但我们直接渲染各个子模板，它们都 extends user_center_base.html
    let template = match current_page {
        "profile" => "user_profile.html",
        "orders" => "user_orders.html",
        "cart" => "user_cart.html",
        "favorites" => "user_favorites.html",   // 必须添加
        _ => "user_center_base.html",
    };

    let theme_manager = state.theme_manager.read().await;
    match theme_manager.render(template, context).await {
        Ok(html) => ([(axum::http::header::CONTENT_TYPE, "text/html")], html).into_response(),
        Err(e) => {
            eprintln!("渲染用户页面失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "页面渲染失败").into_response()
        }
    }
}

/// 个人资料页
pub async fn user_profile_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if !user_info.is_logged_in {
        return axum::response::Redirect::temporary("/login").into_response();
    }
    render_user_page(&state, &user_info, &lang, &lang_options, "profile", HashMap::new()).await
}

/// 我的订单页
pub async fn user_orders_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if !user_info.is_logged_in {
        return axum::response::Redirect::temporary("/login").into_response();
    }
    render_user_page(&state, &user_info, &lang, &lang_options, "orders", HashMap::new()).await
}

/// 购物车页
pub async fn user_cart_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if !user_info.is_logged_in {
        return axum::response::Redirect::temporary("/login").into_response();
    }
    render_user_page(&state, &user_info, &lang, &lang_options, "cart", HashMap::new()).await
}

/// 我的收藏页
pub async fn user_favorites_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if !user_info.is_logged_in {
        return axum::response::Redirect::temporary("/login").into_response();
    }
    render_user_page(&state, &user_info, &lang, &lang_options, "favorites", HashMap::new()).await
}