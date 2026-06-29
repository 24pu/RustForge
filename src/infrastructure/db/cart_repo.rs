// src/infrastructure/db/cart_repo.rs

use sqlx::{PgPool, Postgres, Transaction, Executor};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

use crate::core::models::cart::*;
use crate::core::CartRepository;

pub struct PostgresCartRepo {
    pool: PgPool,
}

impl PostgresCartRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 辅助：获取或创建购物车
    async fn get_or_create_cart(&self, user_id: Uuid) -> Result<Uuid> {
        // 尝试查询购物车
        if let Some(cart) = sqlx::query!(
            "SELECT id FROM carts WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        {
            return Ok(cart.id);
        }

        // 不存在则创建
        let cart_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO carts (id, user_id) VALUES ($1, $2)",
            cart_id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(cart_id)
    }

    // 获取购物车项（含商品和变体信息）
    async fn get_cart_items_with_products(&self, cart_id: Uuid) -> Result<Vec<CartItemWithProduct>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                ci.id,
                ci.cart_id,
                ci.product_id,
                ci.variant_id,
                ci.quantity,
                ci.price,
                ci.created_at,
                ci.updated_at,
                p.name as product_name,
                p.cover_image as product_cover_image,
                pv.sku as variant_sku,
                pv.color as variant_color,
                pv.size as variant_size
            FROM cart_items ci
            JOIN products p ON p.id = ci.product_id
            LEFT JOIN product_variants pv ON pv.id = ci.variant_id
            WHERE ci.cart_id = $1
            ORDER BY ci.created_at DESC
            "#,
            cart_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(CartItemWithProduct {
                item: CartItem {
                    id: row.id,
                    cart_id: row.cart_id,
                    product_id: row.product_id,
                    variant_id: row.variant_id,
                    quantity: row.quantity,
                    price: row.price,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
                product_name: row.product_name,
                product_cover_image: row.product_cover_image,
                variant_sku: Some(row.variant_sku),   // 修改此处
                variant_color: row.variant_color,
                variant_size: row.variant_size,
                total: row.price * row.quantity as f64,
            });
        }
        Ok(items)
    }
}

#[async_trait]
impl CartRepository for PostgresCartRepo {
    async fn get_cart(&self, user_id: Uuid) -> Result<CartWithItems> {
        let cart_id = self.get_or_create_cart(user_id).await?;
        let items = self.get_cart_items_with_products(cart_id).await?;

        // 获取 cart 信息（这里我们实际上只需要 id 和 user_id，但为完整，我们查询）
        let cart_row = sqlx::query!(
            "SELECT id, user_id, created_at, updated_at FROM carts WHERE id = $1",
            cart_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CartWithItems {
            cart: Cart {
                id: cart_row.id,
                user_id: cart_row.user_id,
                created_at: cart_row.created_at,
                updated_at: cart_row.updated_at,
            },
            items,
        })
    }

    async fn add_item(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        variant_id: Option<Uuid>,
        quantity: i32,
    ) -> Result<CartItem> {
        if quantity <= 0 {
            return Err(anyhow!("数量必须大于0"));
        }

        let price = if let Some(vid) = variant_id {
            let row = sqlx::query!("SELECT price::DOUBLE PRECISION as price FROM product_variants WHERE id = $1", vid)
                .fetch_optional(&self.pool)
                .await?
                .ok_or_else(|| anyhow!("变体不存在"))?;
            row.price.ok_or_else(|| anyhow!("变体未设置价格"))?
        } else {
            let row = sqlx::query!("SELECT price FROM products WHERE id = $1", product_id)
                .fetch_optional(&self.pool)
                .await?
                .ok_or_else(|| anyhow!("产品不存在"))?;
            let price_str = row.price.ok_or_else(|| anyhow!("产品未设置价格"))?;
            price_str.parse::<f64>().map_err(|_| anyhow!("价格格式错误"))?
        };

        let cart_id = self.get_or_create_cart(user_id).await?;

        // 检查是否已存在相同的商品+变体
        let existing = sqlx::query!(
            "SELECT id, quantity FROM cart_items WHERE cart_id = $1 AND product_id = $2 AND variant_id IS NOT DISTINCT FROM $3",
            cart_id,
            product_id,
            variant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let item = if let Some(row) = existing {
            let new_qty = row.quantity + quantity;
            let updated = sqlx::query!(
                "UPDATE cart_items SET quantity = $1, updated_at = now() WHERE id = $2 RETURNING *",
                new_qty,
                row.id
            )
            .fetch_one(&self.pool)
            .await?;
            CartItem {
                id: updated.id,
                cart_id: updated.cart_id,
                product_id: updated.product_id,
                variant_id: updated.variant_id,
                quantity: updated.quantity,
                price: updated.price,
                created_at: updated.created_at,
                updated_at: updated.updated_at,
            }
        } else {
            let id = Uuid::new_v4();
            let inserted = sqlx::query!(
                "INSERT INTO cart_items (id, cart_id, product_id, variant_id, quantity, price) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
                id,
                cart_id,
                product_id,
                variant_id,
                quantity,
                price
            )
            .fetch_one(&self.pool)
            .await?;
            CartItem {
                id: inserted.id,
                cart_id: inserted.cart_id,
                product_id: inserted.product_id,
                variant_id: inserted.variant_id,
                quantity: inserted.quantity,
                price: inserted.price,
                created_at: inserted.created_at,
                updated_at: inserted.updated_at,
            }
        };

        Ok(item)
    }

    async fn update_item(&self, user_id: Uuid, item_id: Uuid, quantity: i32) -> Result<CartItem> {
        if quantity <= 0 {
            return Err(anyhow!("数量必须大于0"));
        }

        // 先验证该购物车项是否属于该用户
        let row = sqlx::query!(
            "SELECT ci.id FROM cart_items ci JOIN carts c ON c.id = ci.cart_id WHERE ci.id = $1 AND c.user_id = $2",
            item_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        if row.is_none() {
            return Err(anyhow!("购物车项不存在或不属于该用户"));
        }

        let updated = sqlx::query!(
            "UPDATE cart_items SET quantity = $1, updated_at = now() WHERE id = $2 RETURNING *",
            quantity,
            item_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CartItem {
            id: updated.id,
            cart_id: updated.cart_id,
            product_id: updated.product_id,
            variant_id: updated.variant_id,
            quantity: updated.quantity,
            price: updated.price,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        })
    }

    async fn remove_item(&self, user_id: Uuid, item_id: Uuid) -> Result<bool> {
        let res = sqlx::query!(
            "DELETE FROM cart_items WHERE id = $1 AND cart_id IN (SELECT id FROM carts WHERE user_id = $2)",
            item_id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    async fn clear_cart(&self, user_id: Uuid) -> Result<bool> {
        let res = sqlx::query!(
            "DELETE FROM cart_items WHERE cart_id IN (SELECT id FROM carts WHERE user_id = $1)",
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0 || true) // 即使没有项也返回true
    }

    async fn get_cart_count(&self, user_id: Uuid) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COALESCE(SUM(quantity), 0) as count FROM cart_items WHERE cart_id IN (SELECT id FROM carts WHERE user_id = $1)",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count.unwrap_or(0))
    }
}