use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PluginHook {
    pub id: i32,
    pub plugin_name: String,
    pub hook_name: String,
    pub content: String,
    pub sort_order: Option<i32>,
    pub lang: Option<String>,
    pub enabled: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePluginHookRequest {
    pub plugin_name: String,
    pub hook_name: String,
    pub content: String,
    pub sort_order: Option<i32>,
    pub lang: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePluginHookRequest {
    pub content: Option<String>,
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
    pub lang: Option<String>,
}