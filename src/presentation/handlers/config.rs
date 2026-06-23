// handlers/config.rs

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;
use sqlx::{PgPool, Transaction, Postgres};
use crate::presentation::AppState;
use crate::presentation::types::{UpdateConfigRequest, ConfigResponse};

// 更新配置的函数 - 支持事务
async fn update_config_value(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>, 
    key: &str, 
    value: &str
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        "INSERT INTO site_config (key, value) VALUES ($1, $2) 
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()",
        key, value
    )
    .execute(executor)
    .await?;
    Ok(())
}

// 获取配置
pub async fn get_config_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 从数据库读取配置
    let configs = match sqlx::query!("SELECT key, value FROM site_config")
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("获取配置失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "获取配置失败"})),
            ).into_response();
        }
    };

    let mut config_map = std::collections::HashMap::new();
    for row in configs {
        config_map.insert(row.key, row.value);
    }

    let response = ConfigResponse {
        site_name: config_map.get("site_name").cloned().unwrap_or_default(),
        default_per_page: config_map.get("default_per_page")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        theme_color: config_map.get("theme_color").cloned().unwrap_or_else(|| "blue".to_string()),
        seo_title: config_map.get("seo_title").cloned().unwrap_or_default(),
        seo_description: config_map.get("seo_description").cloned().unwrap_or_default(),
        seo_keywords: config_map.get("seo_keywords").cloned().unwrap_or_default(),
        logo_url: config_map.get("logo_url").cloned().unwrap_or_default(),
        favicon_url: config_map.get("favicon_url").cloned().unwrap_or_default(),
        site_url: config_map.get("site_url").cloned().unwrap_or_default(),

        allowed_file_types: config_map.get("allowed_file_types").cloned().unwrap_or_else(|| "jpg,jpeg,png,gif,webp,mp4,mp3,pdf".to_string()),
        max_file_size_mb: config_map.get("max_file_size_mb")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
        // 产品设置
        product_allowed_image_types: config_map.get("product_allowed_image_types").cloned().unwrap_or_else(|| "jpg,jpeg,png,gif,webp".to_string()),
        product_max_image_size_mb: config_map.get("product_max_image_size_mb")
            .and_then(|v| v.parse().ok())
            .unwrap_or(5.0),
        product_max_images_count: config_map.get("product_max_images_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        product_auto_thumbnail: config_map.get("product_auto_thumbnail")
            .and_then(|v| v.parse().ok())
            .unwrap_or(true),
        product_thumbnail_width: config_map.get("product_thumbnail_width")
            .and_then(|v| v.parse().ok())
            .unwrap_or(200),
        product_thumbnail_height: config_map.get("product_thumbnail_height")
            .and_then(|v| v.parse().ok())
            .unwrap_or(200),
        product_size_inch: config_map.get("product_size_inch")
        .map(|v| v == "true")
        .unwrap_or(false),
    };

    (StatusCode::OK, Json(response)).into_response()
}

// 更新配置
pub async fn update_config_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    // 开始事务
    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("开始事务失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "保存配置失败"})),
            ).into_response();
        }
    };

    // 保存网站设置
    if let Err(e) = update_config_value(&mut *tx, "site_name", &payload.site_name).await {
        eprintln!("保存 site_name 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "default_per_page", &payload.default_per_page.to_string()).await {
        eprintln!("保存 default_per_page 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "theme_color", &payload.theme_color).await {
        eprintln!("保存 theme_color 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "seo_title", &payload.seo_title).await {
        eprintln!("保存 seo_title 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "seo_description", &payload.seo_description).await {
        eprintln!("保存 seo_description 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "seo_keywords", &payload.seo_keywords).await {
        eprintln!("保存 seo_keywords 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "logo_url", &payload.logo_url).await {
        eprintln!("保存 logo_url 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "favicon_url", &payload.favicon_url).await {
        eprintln!("保存 favicon_url 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "allowed_file_types", &payload.allowed_file_types).await {
        eprintln!("保存 allowed_file_types 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    if let Err(e) = update_config_value(&mut *tx, "max_file_size_mb", &payload.max_file_size_mb.to_string()).await {
        eprintln!("保存 max_file_size_mb 失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {}", e)})),
        ).into_response();
    }
    
    // 保存产品设置
    if let Some(ref value) = payload.product_allowed_image_types {
        if let Err(e) = update_config_value(&mut *tx, "product_allowed_image_types", value).await {
            eprintln!("保存 product_allowed_image_types 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("保存失败: {}", e)})),
            ).into_response();
        }
    }
    
    if let Some(value) = payload.product_max_image_size_mb {
        if let Err(e) = update_config_value(&mut *tx, "product_max_image_size_mb", &value.to_string()).await {
            eprintln!("保存 product_max_image_size_mb 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("保存失败: {}", e)})),
            ).into_response();
        }
    }
    
    if let Some(value) = payload.product_max_images_count {
        if let Err(e) = update_config_value(&mut *tx, "product_max_images_count", &value.to_string()).await {
            eprintln!("保存 product_max_images_count 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("保存失败: {}", e)})),
            ).into_response();
        }
    }
    
    if let Some(value) = payload.product_auto_thumbnail {
        if let Err(e) = update_config_value(&mut *tx, "product_auto_thumbnail", &value.to_string()).await {
            eprintln!("保存 product_auto_thumbnail 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("保存失败: {}", e)})),
            ).into_response();
        }
    }
    
    if let Some(value) = payload.product_thumbnail_width {
        if let Err(e) = update_config_value(&mut *tx, "product_thumbnail_width", &value.to_string()).await {
            eprintln!("保存 product_thumbnail_width 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("保存失败: {}", e)})),
            ).into_response();
        }
    }
    
    if let Some(value) = payload.product_thumbnail_height {
        if let Err(e) = update_config_value(&mut *tx, "product_thumbnail_height", &value.to_string()).await {
            eprintln!("保存 product_thumbnail_height 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("保存失败: {}", e)})),
            ).into_response();
        }
    }

    // 
// 保存产品设置 - 尺码使用英寸
// 尺码使用英寸 - product_size_inch 是 bool，直接使用
if let Err(e) = update_config_value(&mut *tx, "product_size_inch", &payload.product_size_inch.to_string()).await {
    eprintln!("保存 product_size_inch 失败: {}", e);
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("保存失败: {}", e)})),
    ).into_response();
}

// 保存 site_url
if let Err(e) = update_config_value(&mut *tx, "site_url", &payload.site_url.unwrap_or_default()).await {
    eprintln!("保存 site_url 失败: {}", e);
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("保存失败: {}", e)})),
    ).into_response();
}

    // 提交事务
    if let Err(e) = tx.commit().await {
        eprintln!("提交事务失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "保存配置失败"})),
        ).into_response();
    }

    (StatusCode::OK, Json(json!({"success": true, "message": "设置保存成功"}))).into_response()
}