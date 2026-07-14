use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::presentation::AppState;
use crate::core::models::{
    Favorite,
    FavoriteWithContent,
    CreateFavoriteRequest,
    UpdateFavoriteRequest,
};
use crate::core::FavoriteRepository;
use crate::presentation::types::UserInfo;

// ===== 分页参数（支持筛选和排序） =====
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub mark: Option<String>,      // 标记筛选：具体标记值，或 "no_mark" 表示无标记
    pub sort_by: Option<String>,   // 排序：created_at_desc, created_at_asc, title_asc, title_desc
}

// ===== 分页响应 =====
#[derive(Debug, Serialize)]
pub struct PaginatedFavorites {
    pub items: Vec<FavoriteWithContent>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

// ===== 辅助：从 UserInfo 提取 Uuid =====
fn get_user_id(user_info: &UserInfo) -> Result<Uuid, StatusCode> {
    user_info
        .user_id
        .as_ref()
        .ok_or(StatusCode::UNAUTHORIZED)
        .and_then(|s| Uuid::parse_str(s).map_err(|_| StatusCode::BAD_REQUEST))
}

// ===== 处理器 =====

/// 收藏内容（若已收藏则更新标记）
pub async fn favorite_handler(
    State(state): State<Arc<AppState>>,
    Extension(user_info): Extension<UserInfo>,
    Json(req): Json<CreateFavoriteRequest>,
) -> Result<Json<Favorite>, StatusCode> {
    if !user_info.is_logged_in {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user_id = get_user_id(&user_info)?;

    let favorite = state
        .favorite_repo
        .create(user_id, req.content_id, req.mark.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(favorite))
}

/// 取消收藏
pub async fn unfavorite_handler(
    State(state): State<Arc<AppState>>,
    Extension(user_info): Extension<UserInfo>,
    Path(content_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    if !user_info.is_logged_in {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user_id = get_user_id(&user_info)?;

    state
        .favorite_repo
        .delete(user_id, content_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// 更新收藏标记（仅更新标记，不影响收藏状态）
pub async fn update_favorite_mark_handler(
    State(state): State<Arc<AppState>>,
    Extension(user_info): Extension<UserInfo>,
    Path(content_id): Path<Uuid>,
    Json(req): Json<UpdateFavoriteRequest>,
) -> Result<Json<Favorite>, StatusCode> {
    if !user_info.is_logged_in {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user_id = get_user_id(&user_info)?;

    let favorite = state
        .favorite_repo
        .update_mark(user_id, content_id, req.mark.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(favorite))
}

/// 检查当前用户是否收藏了指定内容
pub async fn check_favorite_handler(
    State(state): State<Arc<AppState>>,
    Extension(user_info): Extension<UserInfo>,
    Path(content_id): Path<Uuid>,
) -> Result<Json<bool>, StatusCode> {
    if !user_info.is_logged_in {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user_id = get_user_id(&user_info)?;

    let exists = state
        .favorite_repo
        .find_by_user_and_content(user_id, content_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    Ok(Json(exists))
}

/// 获取当前用户的收藏列表（分页，支持筛选和排序）
pub async fn list_favorites_handler(
    State(state): State<Arc<AppState>>,
    Extension(user_info): Extension<UserInfo>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<PaginatedFavorites>, StatusCode> {
    if !user_info.is_logged_in {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user_id = get_user_id(&user_info)?;

    // 在 handler 中：
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = pagination.offset.unwrap_or(0).max(0);
    let mark_filter = pagination.mark.as_deref();
    let sort_by = pagination.sort_by.as_deref();

    let items = state
        .favorite_repo
        .list_by_user(user_id, limit, offset, mark_filter, sort_by)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = state
        .favorite_repo
        .count_by_user_filtered(user_id, mark_filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PaginatedFavorites {
        items,
        total,
        offset,
        limit,
    }))
}