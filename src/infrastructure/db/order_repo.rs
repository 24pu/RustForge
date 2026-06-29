// src/infrastructure/db/order_repo.rs

use sqlx::{PgPool, Executor};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;

use crate::core::models::cart::*;
use crate::core::OrderRepository;
use crate::core::models::*;

pub struct PostgresOrderRepo {
    pool: PgPool,
}

impl PostgresOrderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 生成订单号：时间戳 + 随机数
    fn generate_order_number() -> String {
        let ts = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let rand = (0..6).map(|_| format!("{:X}", rand::random::<u8>() % 16)).collect::<String>();
        format!("ORD{}{}", ts, rand)
    }

    /// 管理员分页查询所有订单
    pub async fn admin_list_orders(
        &self,
        page: i64,
        per_page: i64,
        status: Option<&str>,
        keyword: Option<&str>,
    ) -> Result<(Vec<Order>, i64)> {
        let offset = (page - 1) * per_page;

        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if let Some(s) = status {
            if !s.is_empty() {
                conditions.push(format!("status = ${}", param_idx));
                params.push(s.to_string());
                param_idx += 1;
            }
        }
        if let Some(kw) = keyword {
            if !kw.is_empty() {
                conditions.push(format!(
                    "(order_number ILIKE $${} OR shipping_name ILIKE $${} OR shipping_phone ILIKE $${})",
                    param_idx, param_idx + 1, param_idx + 2
                ));
                let pattern = format!("%{}%", kw);
                params.push(pattern.clone());
                params.push(pattern.clone());
                params.push(pattern);
                param_idx += 3;
            }
        }

        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 查询总数
        let count_sql = format!(
            "SELECT COUNT(*) FROM orders {}",
            where_clause
        );
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for p in &params {
            count_query = count_query.bind(p);
        }
        let total = count_query.fetch_one(&self.pool).await?;

        // 查询数据
        let sql = format!(
            "SELECT * FROM orders {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_idx,
            param_idx + 1
        );
        let mut query = sqlx::query_as::<_, Order>(&sql);
        for p in &params {
            query = query.bind(p);
        }
        let rows = query
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok((rows, total))
    }

    /// 管理员获取全局统计
    pub async fn admin_get_stats(&self) -> Result<serde_json::Value> {
        let rows = sqlx::query!(
            r#"
            SELECT
                status,
                COUNT(*) as count,
                COALESCE(SUM(total_amount), 0) as total
            FROM orders
            GROUP BY status
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = serde_json::Map::new();
        let mut total_orders = 0;
        let mut total_amount = 0.0;
        for row in rows {
            let status = row.status;
            let count = row.count.unwrap_or(0) as i64;
            let total = row.total.unwrap_or(0.0);
            let obj = serde_json::json!({
                "count": count,
                "total": total
            });
            stats.insert(status, obj);
            total_orders += count;
            total_amount += total;
        }
        stats.insert("total_orders".to_string(), serde_json::json!(total_orders));
        stats.insert("total_amount".to_string(), serde_json::json!(total_amount));

        Ok(serde_json::Value::Object(stats))
    }
}

#[async_trait]
impl OrderRepository for PostgresOrderRepo {
    async fn create_order(&self, user_id: Uuid, req: &CreateOrderRequest) -> Result<OrderWithItems> {
        // 使用事务
        let mut tx = self.pool.begin().await?;

        // 1. 获取购物车项（带商品信息）
        let cart_items = sqlx::query!(
            r#"
            SELECT
                ci.product_id,
                ci.variant_id,
                ci.quantity,
                ci.price,
                p.name as product_name,
                pv.sku as variant_sku
            FROM cart_items ci
            JOIN carts c ON c.id = ci.cart_id
            JOIN products p ON p.id = ci.product_id
            LEFT JOIN product_variants pv ON pv.id = ci.variant_id
            WHERE c.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&mut *tx)
        .await?;

        if cart_items.is_empty() {
            return Err(anyhow!("购物车为空，无法创建订单"));
        }

        // 2. 计算总金额
        let total_amount: f64 = cart_items.iter()
            .map(|row| row.price * row.quantity as f64)
            .sum();

        // 3. 生成订单号
        let order_number = Self::generate_order_number();
        let order_id = Uuid::new_v4();

        // 4. 插入订单
        let order_row = sqlx::query!(
            r#"
            INSERT INTO orders (
                id, user_id, order_number, total_amount, status,
                shipping_address, shipping_phone, shipping_name, note
            ) VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8)
            RETURNING *
            "#,
            order_id,
            user_id,
            order_number,
            total_amount,
            req.shipping_address,
            req.shipping_phone,
            req.shipping_name,
            req.note
        )
        .fetch_one(&mut *tx)
        .await?;

        // 5. 插入订单项
        let mut order_items = Vec::new();
        for item in cart_items {
            let item_id = Uuid::new_v4();
            let total = item.price * item.quantity as f64;
            let inserted = sqlx::query!(
                r#"
                INSERT INTO order_items (
                    id, order_id, product_id, variant_id, product_name, variant_sku,
                    quantity, price, total
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
                "#,
                item_id,
                order_id,
                item.product_id,
                item.variant_id,
                item.product_name,
                item.variant_sku,
                item.quantity,
                item.price,
                total
            )
            .fetch_one(&mut *tx)
            .await?;

            order_items.push(OrderItemWithProduct {
                item: OrderItem {
                    id: inserted.id,
                    order_id: inserted.order_id,
                    product_id: inserted.product_id,
                    variant_id: inserted.variant_id,
                    product_name: inserted.product_name,
                    variant_sku: inserted.variant_sku,
                    quantity: inserted.quantity,
                    price: inserted.price,
                    total: inserted.total,
                    created_at: inserted.created_at,
                },
                product_cover_image: None,
                variant_color: None,
                variant_size: None,
            });
        }

        // 6. 清空购物车
        sqlx::query!(
            "DELETE FROM cart_items WHERE cart_id IN (SELECT id FROM carts WHERE user_id = $1)",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // 7. 提交事务
        tx.commit().await?;

        // 构建返回结果
        let order = Order {
            id: order_row.id,
            user_id: order_row.user_id,
            order_number: order_row.order_number,
            total_amount: order_row.total_amount,
            status: order_row.status,
            shipping_address: order_row.shipping_address,
            shipping_phone: order_row.shipping_phone,
            shipping_name: order_row.shipping_name,
            note: order_row.note,
            paid_at: order_row.paid_at,
            created_at: order_row.created_at,
            updated_at: order_row.updated_at,
        };

        Ok(OrderWithItems { order, items: order_items })
    }

    async fn list_orders(&self, user_id: Uuid) -> Result<Vec<Order>> {
        let rows = sqlx::query!(
            "SELECT * FROM orders WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::new();
        for row in rows {
            orders.push(Order {
                id: row.id,
                user_id: row.user_id,
                order_number: row.order_number,
                total_amount: row.total_amount,
                status: row.status,
                shipping_address: row.shipping_address,
                shipping_phone: row.shipping_phone,
                shipping_name: row.shipping_name,
                note: row.note,
                paid_at: row.paid_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }
        Ok(orders)
    }

    async fn get_order(&self, user_id: Uuid, order_id: Uuid) -> Result<Option<OrderWithItems>> {
        let order_row = sqlx::query!(
            "SELECT * FROM orders WHERE id = $1 AND user_id = $2",
            order_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = order_row {
            let order = Order {
                id: row.id,
                user_id: row.user_id,
                order_number: row.order_number,
                total_amount: row.total_amount,
                status: row.status,
                shipping_address: row.shipping_address,
                shipping_phone: row.shipping_phone,
                shipping_name: row.shipping_name,
                note: row.note,
                paid_at: row.paid_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            // 获取订单项
            let item_rows = sqlx::query!(
                r#"
                SELECT
                    oi.*,
                    p.cover_image as product_cover_image,
                    pv.color as variant_color,
                    pv.size as variant_size
                FROM order_items oi
                LEFT JOIN products p ON p.id = oi.product_id
                LEFT JOIN product_variants pv ON pv.id = oi.variant_id
                WHERE oi.order_id = $1
                "#,
                order_id
            )
            .fetch_all(&self.pool)
            .await?;

            let mut items = Vec::new();
            for ir in item_rows {
                items.push(OrderItemWithProduct {
                    item: OrderItem {
                        id: ir.id,
                        order_id: ir.order_id,
                        product_id: ir.product_id,
                        variant_id: ir.variant_id,
                        product_name: ir.product_name,
                        variant_sku: ir.variant_sku,
                        quantity: ir.quantity,
                        price: ir.price,
                        total: ir.total,
                        created_at: ir.created_at,
                    },
                    product_cover_image: ir.product_cover_image,
                    variant_color: ir.variant_color,
                    variant_size: ir.variant_size,
                });
            }

            Ok(Some(OrderWithItems { order, items }))
        } else {
            Ok(None)
        }
    }

    async fn update_order_status(&self, user_id: Uuid, order_id: Uuid, status: &str) -> Result<Order> {
        let valid_statuses = ["pending", "paid", "shipped", "completed", "cancelled"];
        if !valid_statuses.contains(&status) {
            return Err(anyhow!("无效的订单状态"));
        }

        let row = sqlx::query!(
            "UPDATE orders SET status = $1, updated_at = now() WHERE id = $2 AND user_id = $3 RETURNING *",
            status,
            order_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Order {
                id: row.id,
                user_id: row.user_id,
                order_number: row.order_number,
                total_amount: row.total_amount,
                status: row.status,
                shipping_address: row.shipping_address,
                shipping_phone: row.shipping_phone,
                shipping_name: row.shipping_name,
                note: row.note,
                paid_at: row.paid_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        } else {
            Err(anyhow!("订单不存在或不属于该用户"))
        }
    }

    async fn get_order_stats(&self, user_id: Uuid) -> Result<serde_json::Value> {
        let rows = sqlx::query!(
            r#"
            SELECT
                status,
                COUNT(*) as count,
                COALESCE(SUM(total_amount), 0) as total
            FROM orders
            WHERE user_id = $1
            GROUP BY status
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = serde_json::Map::new();
        for row in rows {
            let status = row.status;
            let count = row.count.unwrap_or(0) as i64;
            let total = row.total.unwrap_or(0.0);
            let obj = serde_json::json!({
                "count": count,
                "total": total
            });
            stats.insert(status, obj);
        }

        let total_orders: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM orders WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let total_amount: f64 = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(total_amount), 0) FROM orders WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0.0);

        stats.insert("total_orders".to_string(), serde_json::json!(total_orders));
        stats.insert("total_amount".to_string(), serde_json::json!(total_amount));

        Ok(serde_json::Value::Object(stats))
    }
}