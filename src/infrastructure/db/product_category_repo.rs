// infrastructure/db/product_category_repo.rs

use sqlx::{PgPool, Row};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::core::models::ProductCategory;
use crate::core::ProductCategoryRepository;

pub struct PostgresProductCategoryRepo {
    pub pool: PgPool,
}

impl PostgresProductCategoryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductCategoryRepository for PostgresProductCategoryRepo {
    async fn create_category(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        parent_id: Option<i32>,
    ) -> Result<ProductCategory> {
        let row = sqlx::query(
            r#"INSERT INTO product_categories (name, slug, description, parent_id, sort, show_in_nav)
               VALUES ($1, $2, $3, $4, 0, true)
               RETURNING id, name, slug, description, parent_id, sort, show_in_nav, created_at, updated_at"#,
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(parent_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ProductCategory {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
            description: row.get("description"),
            parent_id: row.get("parent_id"),
            sort: row.get("sort"),
            show_in_nav: row.get("show_in_nav"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            children: None,
            product_count: None,  // 添加
        })
    }

    async fn update_category(
        &self,
        id: i32,
        name: &str,
        slug: &str,
        description: Option<&str>,
        parent_id: Option<i32>,
    ) -> Result<ProductCategory> {
        let row = sqlx::query(
            r#"UPDATE product_categories
               SET name = $1, slug = $2, description = $3, parent_id = $4, updated_at = NOW()
               WHERE id = $5
               RETURNING id, name, slug, description, parent_id, sort, show_in_nav, created_at, updated_at"#,
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(parent_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ProductCategory {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
            description: row.get("description"),
            parent_id: row.get("parent_id"),
            sort: row.get("sort"),
            show_in_nav: row.get("show_in_nav"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            children: None,
            product_count: None,  // 添加
        })
    }

    async fn delete_category(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM product_categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }


async fn list_categories_tree(&self, parent_id: Option<i32>) -> Result<Vec<ProductCategory>> {
    let rows = sqlx::query(
        r#"SELECT id, name, slug, description, parent_id, sort, show_in_nav, created_at, updated_at
           FROM product_categories
           ORDER BY parent_id NULLS FIRST, sort, id"#,
    )
    .fetch_all(&self.pool)
    .await?;

    let all: Vec<ProductCategory> = rows
        .iter()
        .map(|r| ProductCategory {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            description: r.get("description"),
            parent_id: r.get("parent_id"),
            sort: r.get("sort"),
            show_in_nav: r.get("show_in_nav"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            children: None,
            product_count: None,  // 添加
        })
        .collect();

    // 使用传入的 parent_id 参数构建树
    Ok(build_product_category_tree(all, parent_id))
}

    async fn get_category_by_id(&self, id: i32) -> Result<Option<ProductCategory>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, description, parent_id, sort, show_in_nav, created_at, updated_at
               FROM product_categories WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ProductCategory {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            description: r.get("description"),
            parent_id: r.get("parent_id"),
            sort: r.get("sort"),
            show_in_nav: r.get("show_in_nav"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            children: None,
            product_count: None,  // 添加
        }))
    }

    async fn get_category_by_slug(&self, slug: &str) -> Result<Option<ProductCategory>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, description, parent_id, sort, show_in_nav, created_at, updated_at
               FROM product_categories WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ProductCategory {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            description: r.get("description"),
            parent_id: r.get("parent_id"),
            sort: r.get("sort"),
            show_in_nav: r.get("show_in_nav"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            children: None,
            product_count: None,  // 添加
        }))
    }
}

// 构建分类树
fn build_product_category_tree(
    categories: Vec<ProductCategory>,
    parent_id: Option<i32>,
) -> Vec<ProductCategory> {
    let mut result = Vec::new();
    for cat in categories.iter().filter(|c| c.parent_id == parent_id) {
        let mut node = cat.clone();
        node.children = Some(build_product_category_tree(categories.clone(), Some(node.id)));
        result.push(node);
    }
    result
}