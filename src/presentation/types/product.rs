// types/product.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};




// 产品更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub sku: Option<String>,
    pub dname: Option<String>,
    pub fullname: Option<String>,
    pub brand: Option<String>,
    pub cover_image: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub points: Option<String>,
    pub dnote: Option<String>,
    pub csize: Option<String>,
    pub ussize: Option<String>,
    pub asize: Option<String>,
    pub fabric_type: Option<String>,
    pub price: Option<String>,
    pub stock: Option<String>,
    pub package: Option<String>,
    pub weight: Option<String>,
    pub published: Option<bool>,
    pub size_list: Option<String>,
    pub color_list: Option<String>,
    pub color_names: Option<String>,
    pub category_ids: Option<Vec<i32>>,  // 改为数组，支持多选
}

// 产品响应
#[derive(Debug, Serialize, Clone)]
pub struct ProductResponse {
    pub id: Uuid,
    pub slug: String,
    pub sku: String,
    pub lang: Option<String>,
    pub name: String,
    pub dname: Option<String>,
    pub fullname: Option<String>,
    pub brand: Option<String>,
    pub cover_image: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub points: Option<String>,
    pub dnote: Option<String>,
    pub csize: Option<String>,
    pub ussize: Option<String>,
    pub asize: Option<String>,
    pub fabric_type: Option<String>,
    pub price: Option<String>,
    pub stock: Option<String>,
    pub package: Option<String>,
    pub weight: Option<String>,
    pub published: Option<bool>,
    pub translation_group: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub size_list: Option<String>,
    pub color_list: Option<String>,
    pub color_names: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
   
}

// 变体生成请求
#[derive(Debug, Deserialize)]
pub struct GenerateVariantsRequest {
    pub colors: Vec<ColorInfo>,
    pub sizes: Vec<String>,
    pub default_price: Option<f64>,
}

// 颜色信息
#[derive(Debug, Deserialize)]
pub struct ColorInfo {
    pub code: String,
    pub name: String,
}

// 产品创建请求
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub slug: String,
    pub sku: String,
    pub lang: Option<String>,
    pub name: String,
    pub dname: Option<String>,
    pub fullname: Option<String>,
    pub brand: Option<String>,
    pub cover_image: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub points: Option<String>,
    pub dnote: Option<String>,
    pub csize: Option<String>,
    pub ussize: Option<String>,
    pub asize: Option<String>,
    pub fabric_type: Option<String>,
    pub price: Option<String>,
    pub stock: Option<String>,
    pub package: Option<String>,
    pub weight: Option<String>,
    pub published: Option<bool>,
    pub translation_group: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub size_list: Option<String>,
    pub color_list: Option<String>,
    pub color_names: Option<String>,
    pub category_ids: Option<Vec<i32>>,  // 改为数组，支持多选
}