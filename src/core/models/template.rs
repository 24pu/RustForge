use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmaTemplate {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub is_used: bool,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AttributeTemplate {
    pub id: i32,
    pub name: String,
    pub title: Option<String>,
    pub value: Option<String>,
    pub is_used: Option<bool>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AttributeGroup {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Option<Uuid>,
    pub is_used: Option<bool>,
    pub sort_order: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupTemplateRelation {
    pub group_id: i32,
    pub attribute_template_id: i32,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductAttributeValue {
    pub product_id: Uuid,
    pub attribute_template_id: i32,
    pub value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GroupDetail {
    #[serde(flatten)]
    pub group: AttributeGroup,
    pub templates: Vec<AttributeTemplate>,
}