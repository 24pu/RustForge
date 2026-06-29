use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: i32,
    pub filename: String,
    pub storage_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub extension: String,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub folder_id: Option<i32>,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFolder {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MediaFolder>>,
}