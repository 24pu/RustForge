use axum::http::StatusCode;
use uuid::Uuid;
use std::sync::Arc;
use std::path::Path;
use image;
use anyhow::Result;

use crate::core::UserRepository;
use crate::presentation::AppState;
use crate::core::models::Category;

use std::collections::HashMap;
use sqlx::PgPool;
use serde_json::Value;

pub async fn check_permission(
    user_id: Option<Uuid>,
    repo: &Arc<dyn UserRepository>,
    perm: &str,
) -> Result<(), (StatusCode, &'static str)> {
    let uid = user_id.ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))?;
    let has = repo
        .user_has_permission(uid, perm)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"))?;
    if !has {
        Err((StatusCode::FORBIDDEN, "Forbidden"))
    } else {
        Ok(())
    }
}

pub async fn get_config_value(pool: &sqlx::PgPool, key: &str) -> Result<String, String> {
    match sqlx::query!("SELECT value FROM site_config WHERE key = $1", key)
        .fetch_one(pool)
        .await
    {
        Ok(row) => Ok(row.value),
        Err(sqlx::Error::RowNotFound) => Err(format!("Config key '{}' not found", key)),
        Err(e) => {
            eprintln!("Failed to get config value: {}", e);
            Err("Database error".to_string())
        }
    }
}

pub async fn get_nav_categories(state: &Arc<AppState>) -> Vec<Category> {
    let categories = state.content_repo.list_categories_tree(None).await.unwrap_or_default();
    categories.into_iter()
        .filter(|c| c.parent_id.is_none() && c.show_in_nav)
        .map(|mut c| {
            if let Some(children) = &mut c.children {
                children.retain(|child| child.show_in_nav);
            }
            c
        })
        .collect()
}

pub fn generate_thumbnail(src: &Path, dst: &Path, max_size: u32) -> Result<()> {
    let img = image::open(src)?;
    let (width, height) = (img.width(), img.height());
    let (nw, nh) = if width > height {
        (max_size, (max_size as f32 * height as f32 / width as f32) as u32)
    } else {
        ((max_size as f32 * width as f32 / height as f32) as u32, max_size)
    };
    let thumbnail = img.resize(nw, nh, image::imageops::FilterType::Lanczos3);
    thumbnail.save(dst)?;
    Ok(())
}



pub async fn get_site_config_map(pool: &PgPool) -> HashMap<String, String> {
    let mut config = HashMap::new();
    let rows = match sqlx::query!("SELECT key, value FROM site_config")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch site config: {}", e);
            return config;
        }
    };
    for row in rows {
        config.insert(row.key, row.value);
    }
    // 设置默认值
    config.entry("seo_title".to_string()).or_insert("企业网站".to_string());
    config.entry("seo_description".to_string()).or_insert("".to_string());
    config.entry("seo_keywords".to_string()).or_insert("".to_string());
    config.entry("logo_url".to_string()).or_insert("".to_string());
    config.entry("favicon_url".to_string()).or_insert("/favicon.ico".to_string());
    config.entry("site_name".to_string()).or_insert("Enterprise".to_string());
    config
}