use axum::{extract::{State, Path, Query}, Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;

use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::core::models::Content;
use uuid::Uuid;
use crate::presentation::middleware::CurrentUser;

pub async fn list_contents_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListContentsParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let category_id = params.category_id;
    let keyword = params.keyword.as_deref();

    let (total, mut items) = if let Some(cat_id) = category_id {
        let mut total_query = String::from(
            "SELECT COUNT(DISTINCT c.id) as count
             FROM contents c
             JOIN content_categories cc ON c.id = cc.content_id
             WHERE cc.category_id = $1"
        );
        let mut list_query = String::from(
            "SELECT c.id, c.slug, c.title, c.body, c.published, c.cover_image, c.lang, c.translation_group, c.created_at, c.updated_at
             FROM contents c
             JOIN content_categories cc ON c.id = cc.content_id
             WHERE cc.category_id = $1"
        );

        if let Some(kw) = keyword {
            let like = format!("%{}%", kw);
            total_query.push_str(" AND (c.title ILIKE $2 OR c.body ILIKE $2 OR c.slug ILIKE $2)");
            list_query.push_str(" AND (c.title ILIKE $2 OR c.body ILIKE $2 OR c.slug ILIKE $2)");
            list_query.push_str(" ORDER BY c.created_at DESC LIMIT $3 OFFSET $4");

            let total_row = match sqlx::query(&total_query)
                .bind(cat_id)
                .bind(&like)
                .fetch_one(&state.db_pool)
                .await
            {
                Ok(row) => row,
                Err(e) => {
                    eprintln!("Failed to count contents by category: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch count" }))).into_response();
                }
            };
            let total: i64 = total_row.get("count");

            let rows = match sqlx::query(&list_query)
                .bind(cat_id)
                .bind(&like)
                .bind(per_page as i64)
                .bind(offset as i64)
                .fetch_all(&state.db_pool)
                .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    eprintln!("Failed to list contents by category: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch contents" }))).into_response();
                }
            };
            let items = rows.iter().map(|r| Content {
                id: r.get("id"),
                slug: r.get("slug"),
                title: r.get("title"),
                body: r.get("body"),
                cover_image: r.get("cover_image"),
                published: r.get("published"),
                lang: r.get("lang"),
                translation_group: r.get("translation_group"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                categories: vec![],
            }).collect();
            (total, items)
        } else {
            list_query.push_str(" ORDER BY c.created_at DESC LIMIT $2 OFFSET $3");

            let total_row = match sqlx::query(&total_query)
                .bind(cat_id)
                .fetch_one(&state.db_pool)
                .await
            {
                Ok(row) => row,
                Err(e) => {
                    eprintln!("Failed to count contents by category: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch count" }))).into_response();
                }
            };
            let total: i64 = total_row.get("count");

            let rows = match sqlx::query(&list_query)
                .bind(cat_id)
                .bind(per_page as i64)
                .bind(offset as i64)
                .fetch_all(&state.db_pool)
                .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    eprintln!("Failed to list contents by category: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch contents" }))).into_response();
                }
            };
            let items = rows.iter().map(|r| Content {
                id: r.get("id"),
                slug: r.get("slug"),
                title: r.get("title"),
                body: r.get("body"),
                cover_image: r.get("cover_image"),
                published: r.get("published"),
                lang: r.get("lang"),
                translation_group: r.get("translation_group"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                categories: vec![],
            }).collect();
            (total, items)
        }
    } else if let Some(kw) = keyword {
        let like = format!("%{}%", kw);
        let total_row = match sqlx::query!(
            "SELECT COUNT(*) as count FROM contents WHERE title ILIKE $1 OR body ILIKE $1 OR slug ILIKE $1",
            like
        )
        .fetch_one(&state.db_pool)
        .await
        {
            Ok(row) => row,
            Err(e) => {
                eprintln!("Failed to count by keyword: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch count" }))).into_response();
            }
        };
        let total = total_row.count.unwrap_or(0);
        let rows = match sqlx::query!(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at
             FROM contents
             WHERE title ILIKE $1 OR body ILIKE $1 OR slug ILIKE $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
            like, per_page as i64, offset as i64
        )
        .fetch_all(&state.db_pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                eprintln!("Failed to list by keyword: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch contents" }))).into_response();
            }
        };
        let items = rows.into_iter().map(|r| Content {
            id: r.id,
            slug: r.slug,
            title: r.title,
            body: r.body,
            published: r.published,
            cover_image: r.cover_image,
            lang: r.lang,
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
        }).collect();
        (total, items)
    } else {
        let total = match state.content_repo.count_all().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to count contents: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch count" }))).into_response();
            }
        };
        let items = match state.content_repo.list_all(per_page as i64, offset as i64).await {
            Ok(items) => items,
            Err(e) => {
                eprintln!("Failed to list contents: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch contents" }))).into_response();
            }
        };
        (total, items)
    };

    for content in &mut items {
        let categories = state.content_repo.get_content_categories(content.id).await.unwrap_or_default();
        content.categories = categories;
    }

    let total_pages = (total + per_page as i64 - 1) / per_page as i64;
    let resp = json!({
        "items": items,
        "total": total,
        "total_pages": total_pages,
        "current_page": page,
    });
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn list_published_contents_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListContentsParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let keyword = params.keyword.as_deref();
    let lang = params.lang.as_deref().unwrap_or("zh");

    if let Some(kw) = keyword {
        match state.content_repo.search_published(kw, lang, per_page as i64, offset as i64).await {
            Ok((items, total)) => {
                let total_pages = (total + per_page as i64 - 1) / per_page as i64;
                let resp = json!({
                    "items": items,
                    "total": total,
                    "total_pages": total_pages,
                    "current_page": page,
                });
                (StatusCode::OK, Json(resp)).into_response()
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Search failed"}))).into_response(),
        }
    } else {
        // 无关键词时返回最新内容（也可以按语言过滤，但当前保持原样）
        match state.content_repo.list_published(per_page as i64).await {
            Ok(contents) => (StatusCode::OK, Json(contents)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to fetch contents"}))).into_response(),
        }
    }
}

pub async fn create_content_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateContentRequest>,
) -> impl IntoResponse {
    let published = payload.published.unwrap_or(false);
    let lang = payload.lang.unwrap_or_else(|| "zh".to_string());
    let translation_group = payload.translation_group.unwrap_or_else(Uuid::new_v4);
    
    let content = match state.content_repo.create_content(
        &payload.slug, &payload.title, &payload.body, published,
        payload.cover_image.clone(), &lang, translation_group,
    ).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create content: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create content" }))).into_response();
        }
    };
    if let Some(cat_ids) = payload.category_ids {
        if let Err(e) = state.content_repo.set_content_categories(content.id, &cat_ids).await {
            eprintln!("Failed to set categories for content {}: {}", content.id, e);
        }
    }
    (StatusCode::CREATED, Json(content)).into_response()
}

pub async fn update_content_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateContentRequest>,
) -> impl IntoResponse {
    let existing = match state.content_repo.get_content_by_id(id).await {
        Ok(Some(c)) => c,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Content not found" }))).into_response(),
    };
    let published = payload.published.unwrap_or(existing.published);
    let content = match state.content_repo.update_content(
        id, &payload.title, &payload.body, published,
        payload.cover_image.clone(),
    ).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to update content: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to update content" }))).into_response();
        }
    };
    if let Some(cat_ids) = payload.category_ids {
        if let Err(e) = state.content_repo.set_content_categories(id, &cat_ids).await {
            eprintln!("Failed to update categories for content {}: {}", id, e);
        }
    }
    (StatusCode::OK, Json(content)).into_response()
}

pub async fn delete_content_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.content_repo.delete_content(id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Content not found" }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Delete failed" }))).into_response(),
    }
}

pub async fn get_content_by_id_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.content_repo.get_content_by_id(id).await {
        Ok(Some(mut content)) => {
            let categories = state.content_repo.get_content_categories(id).await.unwrap_or_default();
            content.categories = categories;
            (StatusCode::OK, Json(content)).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Content not found" }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch content" }))).into_response(),
    }
}