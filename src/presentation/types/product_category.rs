// types/product_category.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateProductCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}