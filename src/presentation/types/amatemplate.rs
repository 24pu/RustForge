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

