// handlers/home.rs

use axum::{
    extract::{State, Path},
    response::{Html, IntoResponse},
    http::StatusCode,
    Extension,
};
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use crate::presentation::AppState;
use crate::presentation::handlers::utils::{get_nav_categories, get_site_config_map};
use crate::core::ThemeManager;
use crate::presentation::types::UserInfo;
use crate::infrastructure::i18n::LangOption;
use crate::core::{Pagination, ProductFilters};

// 产品列表页
pub async fn products_page_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;
    
    let mut context = HashMap::new();
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
    }));
    context.insert("lang".to_string(), json!(lang));
    context.insert("lang_options".to_string(), json!(state.i18n.lang_options()));
    
    match state.theme_manager.read().await.render("products.html", context).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Render error: {}", e)).into_response(),
    }
}

// 产品详情页


// 通过 slug 获取产品详情
pub async fn product_detail_page_handler_by_slug(
    Extension(user_info): Extension<UserInfo>,
    Path(slug): Path<String>,
    Extension(lang): Extension<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let product = match state.product_repo.get_product_by_slug(&slug).await {
        Ok(Some(p)) => p,
        _ => {
            return (StatusCode::NOT_FOUND, "Product not found").into_response();
        }
    };
    
    let variants = state.product_repo.list_variants(product.id).await.unwrap_or_default();
    let images = state.product_repo.get_product_images(product.id, None).await.unwrap_or_default();
    
    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;
    
    let mut product_json = serde_json::to_value(&product).unwrap_or(json!({}));
    if let Some(obj) = product_json.as_object_mut() {
        obj.insert("images".to_string(), json!(images));
    }
    
    let mut context = HashMap::new();
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("product".to_string(), product_json);
    context.insert("variants".to_string(), json!(variants));
    context.insert("user_info".to_string(), json!({
        "is_logged_in": user_info.is_logged_in,
        "user_name": user_info.user_name,
    }));
    context.insert("lang".to_string(), json!(lang));
    context.insert("lang_options".to_string(), json!(state.i18n.lang_options()));
    
    match state.theme_manager.read().await.render("product.html", context).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Render error: {}", e)).into_response(),
    }
}

// 辅助函数：生成缩略图 URL（与 product_public.rs 保持一致）
fn get_thumbnail_url(original_url: &str) -> Option<String> {
    if original_url.is_empty() {
        return None;
    }
    if original_url.contains("/thumb_") {
        return Some(original_url.to_string());
    }
    if let Some(last_slash) = original_url.rfind('/') {
        let (dir, filename) = original_url.split_at(last_slash + 1);
        if !filename.is_empty() {
            return Some(format!("{}thumb_{}", dir, filename));
        }
    }
    None
}

// 原有的 home_handler（修改后）
pub async fn home_handler(
    Extension(user_info): Extension<UserInfo>,
    Extension(lang): Extension<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 获取最新内容，过滤当前语言
    let latest = state.content_repo.list_published(6).await.unwrap_or_default();
    let filtered_latest: Vec<_> = latest.into_iter()
        .filter(|c| c.lang == lang || c.lang.is_empty())
        .collect();
    
    // 获取热门产品（已发布的前8个），并添加缩略图
    let hot_products = match state.product_repo.list_products(
        Pagination::new(1, 8),
        ProductFilters {
            keyword: None,
            category_id: None,
            published: Some(true),
        }
    ).await {
        Ok((products, _)) => {
            let mut products_with_thumb = Vec::new();
            for p in products {
                let mut json = serde_json::to_value(&p).unwrap_or(json!({}));
                // 添加 thumbnail 字段
                if let Some(cover) = p.cover_image.as_deref() {
                    if let Some(thumb) = get_thumbnail_url(cover) {
                        json["thumbnail"] = json!(thumb);
                    } else {
                        json["thumbnail"] = json!(cover);
                    }
                } else {
                    json["thumbnail"] = json!(null);
                }
                products_with_thumb.push(json);
            }
            products_with_thumb
        },
        Err(e) => {
            eprintln!("获取热门产品失败: {}", e);
            vec![]
        }
    };
    
    // 获取产品分类（带产品数量）
    let product_categories = match state.product_category_repo.list_categories_tree(None).await {
        Ok(cats) => {
            let mut categories_with_count = Vec::new();
            for cat in cats {
                let count = sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM product_category_relations WHERE category_id = $1",
                    cat.id
                )
                .fetch_one(&state.db_pool)
                .await
                .unwrap_or(Some(0))
                .unwrap_or(0);
                
                categories_with_count.push(json!({
                    "id": cat.id,
                    "name": cat.name,
                    "slug": cat.slug,
                    "description": cat.description,
                    "parent_id": cat.parent_id,
                    "sort": cat.sort,
                    "show_in_nav": cat.show_in_nav,
                    "created_at": cat.created_at,
                    "updated_at": cat.updated_at,
                    "product_count": count,
                    "children": cat.children,
                }));
            }
            categories_with_count
        }
        Err(e) => {
            eprintln!("获取产品分类失败: {}", e);
            vec![]
        }
    };
    
    let nav_categories = get_nav_categories(&state).await;
    let site_config = get_site_config_map(&state.db_pool).await;

    let mut context = HashMap::new();
    context.insert("site_config".to_string(), json!(site_config));
    context.insert("latest_contents".to_string(), json!(filtered_latest));
    context.insert("nav_categories".to_string(), json!(nav_categories));
    context.insert("hot_products".to_string(), json!(hot_products));
    context.insert("product_categories".to_string(), json!(product_categories));
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