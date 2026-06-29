use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::models::cart::*;
use crate::core::{CartRepository, OrderRepository};
use crate::infrastructure::db::{PostgresCartRepo, PostgresOrderRepo};
use crate::presentation::middleware::CurrentUser;
use crate::presentation::AppState;
use crate::core::models::*;
use crate::presentation::handlers::utils::check_permission;


// 用户中心页面处理器

// ========== 购物车 Handler ==========

/// 获取购物车
pub async fn get_cart_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresCartRepo::new(state.db_pool.clone());
    match repo.get_cart(user_id).await {
        Ok(cart) => (StatusCode::OK, Json(cart)).into_response(),
        Err(e) => {
            eprintln!("获取购物车失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "获取购物车失败").into_response()
        }
    }
}

/// 添加商品到购物车
pub async fn add_to_cart_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddToCartRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresCartRepo::new(state.db_pool.clone());
    match repo.add_item(user_id, payload.product_id, payload.variant_id, payload.quantity).await {
        Ok(item) => (StatusCode::CREATED, Json(json!({ "success": true, "item": item }))).into_response(),
        Err(e) => {
            eprintln!("添加到购物车失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "添加到购物车失败").into_response()
        }
    }
}

/// 更新购物车项数量
pub async fn update_cart_item_handler(
    CurrentUser(user_id): CurrentUser,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateCartItemRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresCartRepo::new(state.db_pool.clone());
    match repo.update_item(user_id, item_id, payload.quantity).await {
        Ok(item) => (StatusCode::OK, Json(item)).into_response(),
        Err(e) => {
            eprintln!("更新购物车项失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "更新购物车项失败").into_response()
        }
    }
}

/// 删除购物车项
pub async fn remove_cart_item_handler(
    CurrentUser(user_id): CurrentUser,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresCartRepo::new(state.db_pool.clone());
    match repo.remove_item(user_id, item_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "购物车项不存在").into_response(),
        Err(e) => {
            eprintln!("删除购物车项失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "删除购物车项失败").into_response()
        }
    }
}

/// 清空购物车
pub async fn clear_cart_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresCartRepo::new(state.db_pool.clone());
    match repo.clear_cart(user_id).await {
    Ok(true) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
    Ok(false) => (StatusCode::OK, Json(json!({ "success": false, "message": "购物车已空" }))).into_response(),
        Err(e) => {
            eprintln!("清空购物车失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "清空购物车失败").into_response()
        }
    }
}

/// 获取购物车数量
pub async fn get_cart_count_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresCartRepo::new(state.db_pool.clone());
    match repo.get_cart_count(user_id).await {
        Ok(count) => (StatusCode::OK, Json(json!({ "count": count }))).into_response(),
        Err(e) => {
            eprintln!("获取购物车数量失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "获取购物车数量失败").into_response()
        }
    }
}

// ========== 订单 Handler ==========

/// 创建订单（从购物车）
pub async fn create_order_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrderRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresOrderRepo::new(state.db_pool.clone());
    match repo.create_order(user_id, &payload).await {
        Ok(order) => (StatusCode::CREATED, Json(order)).into_response(),
        Err(e) => {
            eprintln!("创建订单失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("创建订单失败: {}", e)).into_response()
        }
    }
}

/// 获取用户订单列表
pub async fn list_orders_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresOrderRepo::new(state.db_pool.clone());
    match repo.list_orders(user_id).await {
        Ok(orders) => (StatusCode::OK, Json(orders)).into_response(),
        Err(e) => {
            eprintln!("获取订单列表失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "获取订单列表失败").into_response()
        }
    }
}

/// 获取订单详情
pub async fn get_order_handler(
    CurrentUser(user_id): CurrentUser,
    Path(order_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresOrderRepo::new(state.db_pool.clone());
    match repo.get_order(user_id, order_id).await {
        Ok(Some(order)) => (StatusCode::OK, Json(order)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "订单不存在").into_response(),
        Err(e) => {
            eprintln!("获取订单详情失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "获取订单详情失败").into_response()
        }
    }
}

/// 更新订单状态（管理员）
pub async fn update_order_status_handler(
    CurrentUser(user_id): CurrentUser,
    Path(order_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateOrderStatusRequest>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    // 仅管理员可操作（需检查权限）
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "order:manage").await {
        return (status, msg).into_response();
    }

    let repo = PostgresOrderRepo::new(state.db_pool.clone());
    match repo.update_order_status(user_id, order_id, &payload.status).await {
        Ok(order) => (StatusCode::OK, Json(order)).into_response(),
        Err(e) => {
            eprintln!("更新订单状态失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("更新订单状态失败: {}", e)).into_response()
        }
    }
}

/// 获取订单统计
pub async fn get_order_stats_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresOrderRepo::new(state.db_pool.clone());
    match repo.get_order_stats(user_id).await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => {
            eprintln!("获取订单统计失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "获取订单统计失败").into_response()
        }
    }
}

/// 取消订单（用户自己取消，仅限 pending 或 paid 状态）
pub async fn cancel_order_handler(
    CurrentUser(user_id): CurrentUser,
    Path(order_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let repo = PostgresOrderRepo::new(state.db_pool.clone());

    match repo.get_order(user_id, order_id).await {
        Ok(Some(order_with_items)) => {
            let order = &order_with_items.order;
            if order.status != "pending" && order.status != "paid" {
                return (
                    StatusCode::BAD_REQUEST,
                    "该订单状态不可取消（仅待支付或已支付可取消）",
                )
                    .into_response();
            }

            match repo.update_order_status(user_id, order_id, "cancelled").await {
                Ok(updated_order) => (StatusCode::OK, Json(updated_order)).into_response(),
                Err(e) => {
                    eprintln!("取消订单失败（更新状态）: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("取消订单失败: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "订单不存在或不属于该用户").into_response(),
        Err(e) => {
            eprintln!("查询订单失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "查询订单失败").into_response()
        }
    }
}