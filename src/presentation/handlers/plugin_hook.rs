// src/presentation/handlers/plugin_hook.rs

use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::presentation::AppState;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::check_permission;
use crate::core::models::{CreatePluginHookRequest, UpdatePluginHookRequest};  // 新增导入
use crate::core::PluginHookRepository;
use crate::infrastructure::db::plugin_hook_repo::PostgresPluginHookRepo;

// ---------- 请求/响应 DTO ----------
#[derive(Debug, serde::Deserialize)]
pub struct ListHookParams {
    pub hook_name: Option<String>,
    pub lang: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateHookRequest {
    pub hook_name: String,
    pub content: String,
    pub sort_order: Option<i32>,
    pub lang: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateHookRequest {
    pub content: Option<String>,
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
    pub lang: Option<String>,  // 新增
}

// ---------- 处理器 ----------



pub async fn list_plugin_hooks(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(plugin_name): Path<String>,
    Query(params): Query<ListHookParams>,
) -> impl IntoResponse {
    // ... 权限检查保持不变 ...
    let user_id = match user_opt {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "plugin:list").await {
        return (status, msg).into_response();
    }

    let repo = PostgresPluginHookRepo::new(state.db_pool.clone());
    
    let hooks = if let Some(hook_name) = params.hook_name {
        // 如果指定了 hook_name，则按钩子名和语言查询
        let lang = params.lang.unwrap_or_else(|| "".to_string());
        match repo.list_by_hook(&hook_name, &lang, params.enabled).await {
            Ok(h) => h,
            Err(e) => {
                eprintln!("查询钩子失败: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "查询失败").into_response();
            }
        }
    } else {
        // 否则返回该插件的所有钩子
        match repo.list_by_plugin(&plugin_name, params.enabled).await {
            Ok(h) => h,
            Err(e) => {
                eprintln!("查询插件钩子失败: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "查询失败").into_response();
            }
        }
    };

    // 无需再过滤，因为 list_by_plugin 已经按插件名查询了
    (StatusCode::OK, Json(hooks)).into_response()
}

pub async fn create_plugin_hook(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(plugin_name): Path<String>,
    Json(payload): Json<CreateHookRequest>,
) -> impl IntoResponse {
    let user_id = match user_opt {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "plugin:install").await {
        return (status, msg).into_response();
    }

    let repo = PostgresPluginHookRepo::new(state.db_pool.clone());
    let req = CreatePluginHookRequest {
        plugin_name: plugin_name.clone(),
        hook_name: payload.hook_name,
        content: payload.content,
        sort_order: payload.sort_order,
        lang: payload.lang,
        enabled: payload.enabled,
    };

    match repo.create(&req).await {
        Ok(hook) => (StatusCode::CREATED, Json(hook)).into_response(),
        Err(e) => {
            eprintln!("创建钩子失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "创建失败").into_response()
        }
    }
}

pub async fn update_plugin_hook(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path((plugin_name, hook_id)): Path<(String, i32)>,
    Json(payload): Json<UpdateHookRequest>,
) -> impl IntoResponse {
    let user_id = match user_opt {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "plugin:install").await {
        return (status, msg).into_response();
    }

    let repo = PostgresPluginHookRepo::new(state.db_pool.clone());

    // 先检查钩子是否存在且属于该插件
    match repo.get_by_id(hook_id).await {
        Ok(Some(hook)) if hook.plugin_name == plugin_name => {
            // 可以更新
            let req = UpdatePluginHookRequest {
                content: payload.content,
                sort_order: payload.sort_order,
                enabled: payload.enabled,
                lang: payload.lang,   // 新增
            };
            match repo.update(hook_id, &req).await {
                Ok(updated) => (StatusCode::OK, Json(updated)).into_response(),
                Err(e) => {
                    eprintln!("更新钩子失败: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "更新失败").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "钩子不存在").into_response(),
        _ => (StatusCode::FORBIDDEN, "无权操作").into_response(),
    }
}

pub async fn delete_plugin_hook(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path((plugin_name, hook_id)): Path<(String, i32)>,
) -> impl IntoResponse {
    let user_id = match user_opt {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "plugin:uninstall").await {
        return (status, msg).into_response();
    }

    let repo = PostgresPluginHookRepo::new(state.db_pool.clone());

    // 检查钩子是否属于该插件
    match repo.get_by_id(hook_id).await {
        Ok(Some(hook)) if hook.plugin_name == plugin_name => {
            match repo.delete(hook_id).await {
                Ok(true) => StatusCode::NO_CONTENT.into_response(),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "删除失败").into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "钩子不存在").into_response(),
        _ => (StatusCode::FORBIDDEN, "无权操作").into_response(),
    }
}