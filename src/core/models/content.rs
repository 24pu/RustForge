use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::category::Category;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub cover_image: Option<String>,
    pub published: bool,
    pub lang: String,
    pub translation_group: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub categories: Vec<Category>,
}