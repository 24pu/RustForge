use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateAmaTemplateRequest {
    pub name: String,
    pub value: String,
    pub is_used: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAmaTemplateRequest {
    pub name: Option<String>,
    pub value: Option<String>,
    pub is_used: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AmaTemplateResponse {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub is_used: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}