use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use async_trait::async_trait;
use crate::core::models::AmaTemplate;
use crate::core::AmaTemplateRepository;

pub struct PostgresAmaTemplateRepo {
    pool: PgPool,
}

impl PostgresAmaTemplateRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl AmaTemplateRepository for PostgresAmaTemplateRepo {
    async fn list(&self, user_id: Uuid) -> Result<Vec<AmaTemplate>> {
        let rows = sqlx::query!(
            "SELECT id, name, value, is_used, user_id, created_at, updated_at FROM amatemplates WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        ).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| AmaTemplate {
            id: r.id,
            name: r.name,
            value: r.value,
            is_used: r.is_used.unwrap_or(false),
            user_id: r.user_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect())
    }

    async fn get_by_id(&self, id: i32, user_id: Uuid) -> Result<Option<AmaTemplate>> {
        let row = sqlx::query!(
            "SELECT id, name, value, is_used, user_id, created_at, updated_at FROM amatemplates WHERE id = $1 AND user_id = $2",
            id, user_id
        ).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| AmaTemplate {
            id: r.id,
            name: r.name,
            value: r.value,
            is_used: r.is_used.unwrap_or(false),
            user_id: r.user_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn create(&self, name: &str, value: &str, is_used: bool, user_id: Uuid) -> Result<AmaTemplate> {
        let row = sqlx::query!(
            "INSERT INTO amatemplates (name, value, is_used, user_id) VALUES ($1, $2, $3, $4) RETURNING id, name, value, is_used, user_id, created_at, updated_at",
            name, value, is_used, user_id
        ).fetch_one(&self.pool).await?;
        Ok(AmaTemplate {
            id: row.id,
            name: row.name,
            value: row.value,
            is_used: row.is_used.unwrap_or(false),
            user_id: row.user_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn update(&self, id: i32, name: &str, value: &str, is_used: bool, user_id: Uuid) -> Result<AmaTemplate> {
        let row = sqlx::query!(
            "UPDATE amatemplates SET name = $1, value = $2, is_used = $3, updated_at = now() WHERE id = $4 AND user_id = $5 RETURNING id, name, value, is_used, user_id, created_at, updated_at",
            name, value, is_used, id, user_id
        ).fetch_one(&self.pool).await?;
        Ok(AmaTemplate {
            id: row.id,
            name: row.name,
            value: row.value,
            is_used: row.is_used.unwrap_or(false),
            user_id: row.user_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete(&self, id: i32, user_id: Uuid) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM amatemplates WHERE id = $1 AND user_id = $2", id, user_id)
            .execute(&self.pool).await?;
        Ok(res.rows_affected() > 0)
    }
}