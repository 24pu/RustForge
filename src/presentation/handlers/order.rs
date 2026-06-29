// src/presentation/handlers/order.rs

use axum::{
    extract::{ State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use axum::extract::Query;
use serde::Deserialize;

use crate::core::{CartRepository, OrderRepository};
use crate::infrastructure::db::{PostgresOrderRepo};
use crate::presentation::middleware::CurrentUser;
use crate::presentation::AppState;
use crate::presentation::handlers::utils::check_permission;

#[derive(Debug, Deserialize)]
pub struct AdminOrderListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

/// 管理员获取所有订单（分页、筛选）
pub async fn admin_list_orders_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<AdminOrderListParams>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    // 权限检查
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "order:list").await {
        return (status, msg).into_response();
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let status = params.status.as_deref();
    let keyword = params.keyword.as_deref();

    let repo = PostgresOrderRepo::new(state.db_pool.clone());
    match repo.admin_list_orders(page, per_page, status, keyword).await {
        Ok((orders, total)) => {
            let total_pages = (total + per_page - 1) / per_page;
            let resp = serde_json::json!({
                "items": orders,
                "total": total,
                "page": page,
                "per_page": per_page,
                "total_pages": total_pages,
            });
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            eprintln!("管理员获取订单列表失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "获取订单列表失败").into_response()
        }
    }
}

/// 管理员获取订单统计
pub async fn admin_order_stats_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "order:list").await {
        return (status, msg).into_response();
    }

    let repo = PostgresOrderRepo::new(state.db_pool.clone());
    match repo.admin_get_stats().await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => {
            eprintln!("管理员获取订单统计失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "获取统计失败").into_response()
        }
    }
}