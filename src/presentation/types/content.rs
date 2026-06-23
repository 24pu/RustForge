// types/content.rs

use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateContentRequest {
    pub slug: String,
    pub title: String,
    pub body: String,
    pub published: Option<bool>,
    pub category_ids: Option<Vec<i32>>,
    pub cover_image: Option<String>,
    pub lang: Option<String>,
    pub translation_group: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateContentRequest {
    pub title: String,
    pub body: String,
    pub published: Option<bool>,
    pub category_ids: Option<Vec<i32>>,
    pub cover_image: Option<String>,
}

#[derive(Deserialize)]
pub struct ListContentsParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub category_id: Option<i32>,
    pub keyword: Option<String>,
    pub lang: Option<String>,
}