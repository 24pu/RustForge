use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// 购物车
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartWithItems {
    #[serde(flatten)]
    pub cart: Cart,
    pub items: Vec<CartItemWithProduct>,
}

// 购物车项
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CartItem {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity: i32,
    pub price: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItemWithProduct {
    #[serde(flatten)]
    pub item: CartItem,
    pub product_name: String,
    pub product_cover_image: Option<String>,
    pub variant_sku: Option<String>,
    pub variant_color: Option<String>,
    pub variant_size: Option<String>,
    pub total: f64,
}

// 请求
#[derive(Debug, Deserialize)]
pub struct AddToCartRequest {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCartItemRequest {
    pub quantity: i32,
}