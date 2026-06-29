use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 产品分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategory {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub sort: i32,
    pub show_in_nav: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub children: Option<Vec<ProductCategory>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_count: Option<i64>,
}

// 产品主表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub slug: String,
    pub lang: String,
    pub sku: Option<String>,
    pub translation_group: Uuid,
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
    pub published: bool,
    pub user_id: Option<Uuid>,
    pub size_list: Option<String>,
    pub color_list: Option<String>,
    pub color_names: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub categories: Vec<ProductCategory>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub variants: Vec<ProductVariant>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub images: Vec<ProductImage>,
}

// 产品变体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub color: Option<String>,
    pub color_code: Option<String>,
    pub color_name: Option<String>,
    pub size: Option<String>,
    pub price: Option<f64>,
    pub stock: i32,
    pub weight: Option<String>,
    pub package_info: Option<String>,
    pub is_default: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 产品图片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductImage {
    pub id: i32,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub url: String,
    pub name: Option<String>,
    pub original_name: Option<String>,
    pub color_code: Option<String>,
    pub image_type: String,
    pub file_size: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}