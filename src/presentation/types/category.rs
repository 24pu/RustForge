// types/category.rs

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub display_type: Option<String>,
    pub show_in_nav: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub display_type: Option<String>,
    pub show_in_nav: Option<bool>,
}

#[derive(Deserialize)]
pub struct ReorderCategoriesRequest {
    pub items: Vec<ReorderItem>,
}

#[derive(Deserialize)]
pub struct ReorderItem {
    pub id: i32,
    pub sort: i32,
}

#[derive(Deserialize)]
pub struct CategoryPageParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}