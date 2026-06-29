use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// 订单
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_number: String,
    pub total_amount: f64,
    pub status: String,
    pub shipping_address: Option<String>,
    pub shipping_phone: Option<String>,
    pub shipping_name: Option<String>,
    pub note: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItemWithProduct>,
}

// 订单项
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub product_name: String,
    pub variant_sku: Option<String>,
    pub quantity: i32,
    pub price: f64,
    pub total: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemWithProduct {
    #[serde(flatten)]
    pub item: OrderItem,
    pub product_cover_image: Option<String>,
    pub variant_color: Option<String>,
    pub variant_size: Option<String>,
}

// 请求
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub shipping_address: String,
    pub shipping_phone: String,
    pub shipping_name: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}