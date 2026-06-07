use axum::{
    extract::{Path, Query, State, Extension},   // 新增 Extension
    response::{Html, IntoResponse},
    http::StatusCode,
};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;
use markdown;

use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::presentation::handlers::utils::{check_permission, get_nav_categories,get_site_config_map};
use crate::core::models::Category;
use uuid::Uuid;
use crate::presentation::middleware::CurrentUser;
use crate::core::ThemeManager;   // 重要：让 render 方法可用
use crate::presentation::types::UserInfo;       // 导入 UserInfo
use crate::infrastructure::i18n::LangOption;

pub async fn list_categories_tree_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.content_repo.list_categories_tree(None).await {
        Ok(tree) => (StatusCode::OK, Json(tree)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch categories" }))).into_response(),
    }
}

pub async fn create_category_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCategoryRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "category:create").await {
        return (status, msg).into_response();
    }
    let display_type = payload.display_type.unwrap_or_else(|| "list".to_string());
    let show_in_nav = payload.show_in_nav.unwrap_or(true);
    match state.content_repo.create_category(
        &payload.name,
        &payload.slug,
        payload.description.as_deref(),
        payload.parent_id,
        &display_type,
        show_in_nav,
    ).await {
        Ok(cat) => (StatusCode::CREATED, Json(cat)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create category: {}", e)).into_response(),
    }
}

pub async fn update_category_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "category:edit").await {
        return (status, msg).into_response();
    }
    let display_type = payload.display_type.unwrap_or_else(|| "list".to_string());
    let show_in_nav = payload.show_in_nav.unwrap_or(true);
    match state.content_repo.update_category(
        id,
        &payload.name,
        &payload.slug,
        payload.description.as_deref(),
        payload.parent_id,
        &display_type,
        show_in_nav,
    ).await {
        Ok(cat) => (StatusCode::OK, Json(cat)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update category: {}", e)).into_response(),
    }
}

pub async fn delete_category_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "category:delete").await {
        return (status, msg).into_response();
    }
    match state.content_repo.delete_category(id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Category not found" }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to delete category" }))).into_response(),
    }
}

pub async fn get_category_by_id_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.content_repo.get_category_by_id(id).await {
        Ok(Some(cat)) => (StatusCode::OK, Json(cat)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Category not found" }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch category" }))).into_response(),
    }
}

pub async fn reorder_categories_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReorderCategoriesRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "category:edit").await {
        return (status, msg).into_response();
    }
    let updates: Vec<(i32, i32)> = payload.items.into_iter().map(|item| (item.id, item.sort)).collect();
    match state.content_repo.update_categories_order(updates).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "message": "Order updated" }))).into_response(),
        Err(e) => {
            eprintln!("Failed to update categories order: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to update order" }))).into_response()
        }
    }
}



pub async fn category_page_handler(
    Extension(user_info): Extension<UserInfo>,   // 新增参数
    Path(slug): Path<String>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    Query(params): Query<CategoryPageParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let category = match state.content_repo.get_category_by_slug(&slug).await {
        Ok(Some(c)) => c,
        _ => return (StatusCode::NOT_FOUND, "Category not found").into_response(),
    };
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(12).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let total = state.content_repo.count_by_category_slug_and_lang(&slug, &lang).await.unwrap_or(0);
    let contents = state.content_repo.list_by_category_slug_and_lang(&slug, &lang, per_page as i64, offset as i64).await.unwrap_or_default();
    let total_pages = (total + per_page as i64 - 1) / per_page as i64;

    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;

    let mut context = HashMap::new();
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
    }));
    context.insert("category".to_string(), json!(category));
    context.insert("contents".to_string(), json!(contents));
    context.insert("current_page".to_string(), json!(page));
    context.insert("total_pages".to_string(), json!(total_pages));
    context.insert("lang".to_string(), json!(lang));                // 新增
    context.insert("lang_options".to_string(), json!(lang_options)); // 新增
    let mut pages = Vec::new();
    for i in 1..=total_pages {
        pages.push(i);
    }
    context.insert("pages".to_string(), json!(pages));

    match state.theme_manager.read().await.render("category.html", context).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}


pub async fn content_detail_handler(
    Extension(user_info): Extension<UserInfo>,
    Path(slug): Path<String>,
    Extension(lang): Extension<String>,
    Extension(lang_options): Extension<Vec<LangOption>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut content = match state.content_repo.get_content_by_slug_public(&slug).await {
        Ok(Some(c)) => c,
        _ => return (StatusCode::NOT_FOUND, "Content not found").into_response(),
    };

    // 获取分类
    let categories = state.content_repo.get_content_categories(content.id).await.unwrap_or_default();
    content.categories = categories.clone();

    // 获取相关文章：基于分类，排除自身，最多5篇
    let category_ids: Vec<i32> = categories.iter().map(|c| c.id).collect();
    let related = state.content_repo.get_related_contents(content.id, &category_ids, 5).await.unwrap_or_default();

    let nav_categories = get_nav_categories(&state).await;
    let body_html = markdown::to_html(&content.body);
    let site_config = get_site_config_map(&state.db_pool).await;

    let mut context = HashMap::new();
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("post".to_string(), json!(content));
    context.insert("body_html".to_string(), json!(body_html));
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
    }));
    context.insert("related".to_string(), json!(related));   // 注入相关文章
    context.insert("lang".to_string(), json!(lang));                // 新增
    context.insert("lang_options".to_string(), json!(lang_options)); // 新增

    match state.theme_manager.read().await.render("content.html", context).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Render error: {}", e)).into_response(),
    }
}