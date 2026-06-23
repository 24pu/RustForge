// handlers/product_public.rs

use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::presentation::AppState;
use crate::core::{ProductRepository, ProductCategoryRepository, Pagination, ProductFilters};
use crate::presentation::handlers::utils::{
    size_table_to_json, 
    generate_size_table_with_unit
};
// 导入 trait
use crate::core::{ProductAttributeValueRepository, AttributeTemplateRepository};
use crate::infrastructure::db::{
    PostgresProductAttributeValueRepo,
    PostgresAttributeTemplateRepo,
};

// ---------- Error Handling ----------
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": self.to_string() })),
        )
            .into_response()
    }
}


// 获取热门产品（前台展示）
pub async fn get_hot_products_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let pagination = Pagination::new(1, 8);
    let filters = ProductFilters {
        keyword: None,
        category_id: None,
        published: Some(true),
    };
    
    match state.product_repo.list_products(pagination, filters).await {
        Ok((products, _)) => {
            (StatusCode::OK, Json(products)).into_response()
        }
        Err(e) => {
            eprintln!("获取热门产品失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

// 获取所有产品（前台分页列表）
pub async fn get_public_products_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let page = params.get("page")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);
    let per_page = params.get("per_page")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(12);
    let keyword = params.get("keyword").cloned();
    let category_id = params.get("category_id")
        .and_then(|c| c.parse::<i32>().ok());
    let sort_by = params.get("sort_by").cloned();
    
    let pagination = Pagination::new(page, per_page);
    let filters = ProductFilters {
        keyword,
        category_id,
        published: Some(true),
    };
    
    match state.product_repo.list_products(pagination, filters).await {
        Ok((products, total)) => {
            let total_pages = (total + per_page as i64 - 1) / per_page as i64;
            let response = json!({
                "products": products,
                "total": total,
                "page": page,
                "per_page": per_page,
                "total_pages": total_pages
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            eprintln!("获取产品列表失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

// 获取单个产品详情（前台）
pub async fn get_public_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.product_repo.get_product_by_id(id).await {
        Ok(Some(product)) => {
            // 获取产品的变体
            let variants = match state.product_repo.list_variants(id).await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("获取变体失败: {}", e);
                    vec![]
                }
            };
            
            let mut product_json = serde_json::to_value(&product).unwrap_or(json!({}));
            if let Some(obj) = product_json.as_object_mut() {
                obj.insert("variants".to_string(), json!(variants));
            }
            
            (StatusCode::OK, Json(product_json)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "产品不存在"})),
        ).into_response(),
        Err(e) => {
            eprintln!("获取产品失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

// 获取产品分类（带产品数量）
pub async fn get_product_categories_with_count_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 获取所有分类
    let categories = match state.product_category_repo.list_categories_tree(None).await {
        Ok(cats) => cats,
        Err(e) => {
            eprintln!("获取分类失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response();
        }
    };
    
    // 获取每个分类的产品数量
    let mut result = Vec::new();
    for cat in categories {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM product_category_relations WHERE category_id = $1",
            cat.id
        )
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);
        
        result.push(json!({
            "id": cat.id,
            "name": cat.name,
            "slug": cat.slug,
            "description": cat.description,
            "parent_id": cat.parent_id,
            "sort": cat.sort,
            "show_in_nav": cat.show_in_nav,
            "product_count": count,
            "children": cat.children,
        }));
    }
    
    (StatusCode::OK, Json(result)).into_response()
}







// 获取产品尺码表





// 获取产品尺码表
pub async fn get_product_size_table_handler(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let product = match state.product_repo.get_product_by_id(product_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "产品不存在"})),
            ).into_response();
        }
        Err(e) => {
            eprintln!("获取产品失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response();
        }
    };
    
    // 从产品的 dnote 字段获取原始尺码数据
    let raw_data = match product.dnote {
        Some(ref data) => data,
        None => {
            return (StatusCode::OK, Json(json!({
                "size_table": null,
                "message": "该产品暂无尺码数据"
            }))).into_response();
        }
    };
    
    // 使用带单位转换的版本
    let size_table = generate_size_table_with_unit(raw_data, &product, &state.db_pool).await;
    
    let json_data = size_table_to_json(&size_table);
    
    (StatusCode::OK, Json(json!({
        "success": true,
        "size_table": json_data,
        "unit": size_table.unit
    }))).into_response()
}

// 获取产品属性值（公开接口）
pub async fn get_public_product_attributes(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let value_repo = PostgresProductAttributeValueRepo::new(state.db_pool.clone());
    let template_repo = PostgresAttributeTemplateRepo::new(state.db_pool.clone());
    
    // 获取产品属性值
    let values = value_repo.get_product_attribute_values(product_id).await?;
    
    // 获取模板名称并附加到返回值
    let mut result = Vec::new();
    for val in values {
        if let Some(template) = template_repo.get_template_by_id(val.attribute_template_id).await? {
            result.push(json!({
                "attribute_template_id": val.attribute_template_id,
                "value": val.value,
                "attribute_name": template.name,
            }));
        }
    }
    
    Ok(Json(result))
}