use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::core::models::Content;   // 导入 Content

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]  // 添加 Serialize
pub struct Favorite {
    pub id: i32,
    pub user_id: Uuid,
    pub content_id: Uuid,           // 改为 Uuid
    pub mark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 请求/响应结构体
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateFavoriteRequest {
    pub content_id: Uuid,           // 改为 Uuid
    pub mark: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateFavoriteRequest {
    pub mark: Option<String>,
}

// 收藏+内容关联（用于列表展示）
#[derive(Debug, Serialize)]
pub struct FavoriteWithContent {
    pub favorite: Favorite,
    pub content: Content,           // 假设 Content 的 id 也是 Uuid
}