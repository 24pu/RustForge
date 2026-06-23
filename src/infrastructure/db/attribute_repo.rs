// src/infrastructure/db/attribute_repo.rs

use sqlx::{PgPool, Transaction, Postgres, Executor, QueryBuilder};
use uuid::Uuid;
use anyhow::Result;
use async_trait::async_trait;
use crate::core::{
    AttributeTemplateRepository, AttributeGroupRepository, ProductAttributeValueRepository,
    CreateAttributeTemplateInput, UpdateAttributeTemplateInput,
    CreateAttributeGroupInput, UpdateAttributeGroupInput,
};
use crate::core::models::{AttributeTemplate, AttributeGroup, ProductAttributeValue};

// ---------- Attribute Template Repository ----------
pub struct PostgresAttributeTemplateRepo {
    pool: PgPool,
}

impl PostgresAttributeTemplateRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl AttributeTemplateRepository for PostgresAttributeTemplateRepo {
    async fn list_templates(&self, is_used: Option<bool>) -> Result<Vec<AttributeTemplate>> {
        let mut query_builder = QueryBuilder::new(
            "SELECT id, name, title,value, is_used, user_id, created_at, updated_at \
             FROM product_attribute_templates"
        );

        if let Some(used) = is_used {
            query_builder.push(" WHERE is_used = ");
            query_builder.push_bind(used);
        }

        query_builder.push(" ORDER BY name");

        let rows = query_builder
            .build_query_as::<AttributeTemplate>()
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get_template_by_id(&self, id: i32) -> Result<Option<AttributeTemplate>> {
        let row = sqlx::query_as!(
            AttributeTemplate,
            "SELECT id, name, title,value, is_used, user_id, created_at, updated_at \
             FROM product_attribute_templates WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create_template(&self, input: CreateAttributeTemplateInput) -> Result<AttributeTemplate> {
        let row = sqlx::query_as!(
            AttributeTemplate,
            "INSERT INTO product_attribute_templates (name, title,value, is_used, user_id) \
             VALUES ($1, $2, $3, $4,$5) \
             RETURNING id, name, title,value, is_used, user_id, created_at, updated_at",
            input.name,
            input.title,
            input.value,
            input.is_used,
            input.user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn update_template(&self, id: i32, input: UpdateAttributeTemplateInput) -> Result<AttributeTemplate> {
        let row = sqlx::query_as!(
            AttributeTemplate,
            "UPDATE product_attribute_templates \
             SET name = COALESCE($1, name), \
                 title = COALESCE($2, title), \
                 value = COALESCE($3, value), \
                 is_used = COALESCE($4, is_used), \
                 updated_at = now() \
             WHERE id = $5 \
             RETURNING id, name, title,value, is_used, user_id, created_at, updated_at",
            input.name,
            input.title,
            input.value,
            input.is_used,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn delete_template(&self, id: i32) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM product_attribute_templates WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}

// ---------- Attribute Group Repository ----------
pub struct PostgresAttributeGroupRepo {
    pool: PgPool,
}

impl PostgresAttributeGroupRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl AttributeGroupRepository for PostgresAttributeGroupRepo {
    async fn list_groups(&self, user_id: Option<Uuid>) -> Result<Vec<AttributeGroup>> {
        let mut query_builder = QueryBuilder::new(
            "SELECT id, name, description, user_id, is_used, sort_order, created_at, updated_at \
             FROM attribute_groups"
        );

        if let Some(uid) = user_id {
            query_builder.push(" WHERE user_id = ");
            query_builder.push_bind(uid);
        }

        query_builder.push(" ORDER BY sort_order, name");

        let rows = query_builder
            .build_query_as::<AttributeGroup>()
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get_group_by_id(&self, id: i32) -> Result<Option<AttributeGroup>> {
        let row = sqlx::query_as!(
            AttributeGroup,
            "SELECT id, name, description, user_id, is_used, sort_order, created_at, updated_at \
             FROM attribute_groups WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create_group(&self, input: CreateAttributeGroupInput) -> Result<AttributeGroup> {
        let row = sqlx::query_as!(
            AttributeGroup,
            "INSERT INTO attribute_groups (name, description, user_id, is_used, sort_order) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, name, description, user_id, is_used, sort_order, created_at, updated_at",
            input.name,
            input.description,
            input.user_id,
            input.is_used,
            input.sort_order
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn update_group(&self, id: i32, input: UpdateAttributeGroupInput) -> Result<AttributeGroup> {
        let row = sqlx::query_as!(
            AttributeGroup,
            "UPDATE attribute_groups \
             SET name = COALESCE($1, name), \
                 description = COALESCE($2, description), \
                 is_used = COALESCE($3, is_used), \
                 sort_order = COALESCE($4, sort_order), \
                 updated_at = now() \
             WHERE id = $5 \
             RETURNING id, name, description, user_id, is_used, sort_order, created_at, updated_at",
            input.name,
            input.description,
            input.is_used,
            input.sort_order,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn delete_group(&self, id: i32) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM attribute_groups WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    // ---------- Group-Template Relations ----------
    async fn get_group_templates(&self, group_id: i32) -> Result<Vec<AttributeTemplate>> {
        let rows = sqlx::query_as!(
            AttributeTemplate,
            r#"
            SELECT at.id, at.name, at.title,at.value, at.is_used, at.user_id, at.created_at, at.updated_at
            FROM product_attribute_templates at
            JOIN group_attribute_template_relations rel ON rel.attribute_template_id = at.id
            WHERE rel.group_id = $1
            ORDER BY rel.sort_order, at.name
            "#,
            group_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn add_template_to_group(&self, group_id: i32, template_id: i32, sort_order: i32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO group_attribute_template_relations (group_id, attribute_template_id, sort_order) \
             VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            group_id,
            template_id,
            sort_order
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_template_from_group(&self, group_id: i32, template_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM group_attribute_template_relations WHERE group_id = $1 AND attribute_template_id = $2",
            group_id,
            template_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_template_sort_in_group(&self, group_id: i32, template_id: i32, sort_order: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE group_attribute_template_relations SET sort_order = $1 \
             WHERE group_id = $2 AND attribute_template_id = $3",
            sort_order,
            group_id,
            template_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ---------- Product Attribute Value Repository ----------
pub struct PostgresProductAttributeValueRepo {
    pool: PgPool,
}

impl PostgresProductAttributeValueRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl ProductAttributeValueRepository for PostgresProductAttributeValueRepo {
    async fn get_product_attribute_values(&self, product_id: Uuid) -> Result<Vec<ProductAttributeValue>> {
        let rows = sqlx::query_as!(
            ProductAttributeValue,
            "SELECT product_id, attribute_template_id, value, created_at, updated_at \
             FROM product_attribute_values WHERE product_id = $1",
            product_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn set_product_attribute_values(&self, product_id: Uuid, values: &[crate::core::ProductAttributeValueInput]) -> Result<()> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        // 先删除该产品所有现有值
        sqlx::query!("DELETE FROM product_attribute_values WHERE product_id = $1", product_id)
            .execute(&mut *tx)
            .await?;

        // 批量插入新值
        for val in values {
            sqlx::query!(
                "INSERT INTO product_attribute_values (product_id, attribute_template_id, value) VALUES ($1, $2, $3)",
                product_id,
                val.attribute_template_id,
                val.value
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn delete_product_attribute_value(&self, product_id: Uuid, attribute_template_id: i32) -> Result<bool> {
        let res = sqlx::query!(
            "DELETE FROM product_attribute_values WHERE product_id = $1 AND attribute_template_id = $2",
            product_id,
            attribute_template_id
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}