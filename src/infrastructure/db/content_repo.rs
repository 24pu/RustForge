use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;
use anyhow::Result;
use async_trait::async_trait;
use crate::core::models::{Content, Category};
use crate::core::ContentRepository;
use crate::core::SitemapEntry;

pub struct PostgresContentRepo {
    pub pool: PgPool,
}

impl PostgresContentRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContentRepository for PostgresContentRepo {
    async fn create_content(
        &self, slug: &str, title: &str, body: &str, published: bool,
        cover_image: Option<String>, lang: &str, translation_group: Uuid,
    ) -> Result<Content> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let row = sqlx::query(
            "INSERT INTO contents (id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at"
        )
        .bind(id)
        .bind(slug)
        .bind(title)
        .bind(body)
        .bind(published)
        .bind(&cover_image)
        .bind(lang)
        .bind(translation_group)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(Content {
            id: row.get("id"),
            slug: row.get("slug"),
            title: row.get("title"),
            body: row.get("body"),
            cover_image: row.get("cover_image"),
            published: row.get("published"),
            lang: row.get("lang"),
            translation_group: row.get("translation_group"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            categories: vec![],
        })
    }

    async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>> {
        let row = sqlx::query(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at FROM contents WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Content {
            id: r.get("id"),
            slug: r.get("slug"),
            title: r.get("title"),
            body: r.get("body"),
            cover_image: r.get("cover_image"),
            published: r.get("published"),
            lang: r.get("lang"),
            translation_group: r.get("translation_group"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            categories: vec![],
        }))
    }

    async fn get_content_by_slug_and_lang(&self, slug: &str, lang: &str) -> Result<Option<Content>> {
        let row = sqlx::query(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at FROM contents WHERE slug = $1 AND lang = $2"
        )
        .bind(slug)
        .bind(lang)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Content {
            id: r.get("id"),
            slug: r.get("slug"),
            title: r.get("title"),
            body: r.get("body"),
            cover_image: r.get("cover_image"),
            published: r.get("published"),
            lang: r.get("lang"),
            translation_group: r.get("translation_group"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            categories: vec![],
        }))
    }

    async fn get_content_translations(&self, translation_group: Uuid, exclude_lang: &str) -> Result<Vec<Content>> {
        let rows = sqlx::query(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at 
             FROM contents WHERE translation_group = $1 AND lang != $2"
        )
        .bind(translation_group)
        .bind(exclude_lang)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| Content {
            id: r.get("id"),
            slug: r.get("slug"),
            title: r.get("title"),
            body: r.get("body"),
            cover_image: r.get("cover_image"),
            published: r.get("published"),
            lang: r.get("lang"),
            translation_group: r.get("translation_group"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            categories: vec![],
        }).collect())
    }

    async fn list_published(&self, limit: i64) -> Result<Vec<Content>> {
        let rows = sqlx::query(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at
             FROM contents WHERE published = true
             ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| Content {
            id: r.get("id"),
            slug: r.get("slug"),
            title: r.get("title"),
            body: r.get("body"),
            cover_image: r.get("cover_image"),
            published: r.get("published"),
            lang: r.get("lang"),
            translation_group: r.get("translation_group"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            categories: vec![],
        }).collect())
    }

    async fn get_content_by_id(&self, id: Uuid) -> Result<Option<Content>> {
        let row = sqlx::query(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at FROM contents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Content {
            id: r.get("id"),
            slug: r.get("slug"),
            title: r.get("title"),
            body: r.get("body"),
            cover_image: r.get("cover_image"),
            published: r.get("published"),
            lang: r.get("lang"),
            translation_group: r.get("translation_group"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            categories: vec![],
        }))
    }

    async fn update_content(&self, id: Uuid, title: &str, body: &str, published: bool, cover_image: Option<String>) -> Result<Content> {
        let now = Utc::now();
        let row = sqlx::query(
            "UPDATE contents SET title = $1, body = $2, published = $3, cover_image = $4, updated_at = $5 WHERE id = $6
             RETURNING id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at"
        )
        .bind(title)
        .bind(body)
        .bind(published)
        .bind(&cover_image)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(Content {
            id: row.get("id"),
            slug: row.get("slug"),
            title: row.get("title"),
            body: row.get("body"),
            cover_image: row.get("cover_image"),
            published: row.get("published"),
            lang: row.get("lang"),
            translation_group: row.get("translation_group"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            categories: vec![],
        })
    }

    async fn delete_content(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM contents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>> {
        let rows = sqlx::query(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at
             FROM contents ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| Content {
            id: r.get("id"),
            slug: r.get("slug"),
            title: r.get("title"),
            cover_image: r.get("cover_image"),
            body: r.get("body"),
            published: r.get("published"),
            lang: r.get("lang"),
            translation_group: r.get("translation_group"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            categories: vec![],
        }).collect())
    }

    async fn count_all(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM contents")
            .fetch_one(&self.pool)
            .await?;
        let count: i64 = row.get("count");
        Ok(count)
    }

    // ---------- 分类管理 ----------
    async fn list_categories_tree(&self, parent_id: Option<i32>) -> Result<Vec<Category>> {
        let rows = sqlx::query!(
            "SELECT id, name, slug, description, parent_id, sort, display_type, show_in_nav, created_at, updated_at
             FROM categories
             ORDER BY parent_id NULLS FIRST, sort, id"
        )
        .fetch_all(&self.pool)
        .await?;
        let all: Vec<Category> = rows.into_iter().map(|r| Category {
            id: r.id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            parent_id: r.parent_id,
            sort: r.sort.unwrap_or(0),
            display_type: r.display_type,
            show_in_nav: r.show_in_nav.unwrap_or(true),
            created_at: r.created_at,
            updated_at: r.updated_at,
            children: None,
        }).collect();
        let tree = build_category_tree(all, parent_id);
        Ok(tree)
    }

    async fn get_category_by_id(&self, id: i32) -> Result<Option<Category>> {
        let row = sqlx::query!(
            "SELECT id, name, slug, sort, description, parent_id, display_type, show_in_nav, created_at, updated_at
             FROM categories WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Category {
            id: r.id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            parent_id: r.parent_id,
            sort: r.sort.unwrap_or(0),
            display_type: r.display_type,
            show_in_nav: r.show_in_nav.unwrap_or(true),
            created_at: r.created_at,
            updated_at: r.updated_at,
            children: None,
        }))
    }

    async fn create_category(&self, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>, display_type: &str, show_in_nav: bool) -> Result<Category> {
        let row = sqlx::query!(
            "INSERT INTO categories (name, slug, description, parent_id, display_type, show_in_nav)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, slug, description, parent_id, sort, display_type, show_in_nav, created_at, updated_at",
            name, slug, description, parent_id, display_type, show_in_nav
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(Category {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            parent_id: row.parent_id,
            sort: row.sort.unwrap_or(0),
            display_type: row.display_type,
            show_in_nav: row.show_in_nav.unwrap_or(true),
            created_at: row.created_at,
            updated_at: row.updated_at,
            children: None,
        })
    }

    async fn update_category(&self, id: i32, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>, display_type: &str, show_in_nav: bool) -> Result<Category> {
        let row = sqlx::query!(
            "UPDATE categories
            SET name = $1, slug = $2, description = $3, parent_id = $4, display_type = $5, show_in_nav = $6, updated_at = now()
            WHERE id = $7
            RETURNING id, name, slug, description, parent_id, sort, display_type, show_in_nav, created_at, updated_at",
            name, slug, description, parent_id, display_type, show_in_nav, id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(Category {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            parent_id: row.parent_id,
            sort: row.sort.unwrap_or(0),
            display_type: row.display_type,
            show_in_nav: row.show_in_nav.unwrap_or(true),
            created_at: row.created_at,
            updated_at: row.updated_at,
            children: None,
        })
    }

    async fn delete_category(&self, id: i32) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM categories WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    // ---------- 内容-分类关联 ----------
    async fn get_content_categories(&self, content_id: Uuid) -> Result<Vec<Category>> {
        let rows = sqlx::query!(
            "SELECT c.id, c.name, c.slug, c.description, c.parent_id, c.sort, c.display_type, c.show_in_nav, c.created_at, c.updated_at
             FROM content_categories cc
             JOIN categories c ON cc.category_id = c.id
             WHERE cc.content_id = $1",
            content_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| Category {
            id: r.id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            parent_id: r.parent_id,
            sort: r.sort.unwrap_or(0),
            display_type: r.display_type,
            show_in_nav: r.show_in_nav.unwrap_or(true),
            created_at: r.created_at,
            updated_at: r.updated_at,
            children: None,
        }).collect())
    }

    async fn set_content_categories(&self, content_id: Uuid, category_ids: &[i32]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!("DELETE FROM content_categories WHERE content_id = $1", content_id)
            .execute(&mut *tx)
            .await?;
        for &cid in category_ids {
            sqlx::query!("INSERT INTO content_categories (content_id, category_id) VALUES ($1, $2)", content_id, cid)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn count_by_category(&self, category_id: i32) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(DISTINCT c.id) as count
             FROM contents c
             JOIN content_categories cc ON c.id = cc.content_id
             WHERE cc.category_id = $1",
            category_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count.unwrap_or(0))
    }

    async fn list_by_category(&self, category_id: i32, limit: i64, offset: i64) -> Result<Vec<Content>> {
        let rows = sqlx::query!(
            "SELECT c.id, c.slug, c.title, c.body, c.published, c.cover_image, c.lang, c.translation_group, c.created_at, c.updated_at
            FROM contents c
            JOIN content_categories cc ON c.id = cc.content_id
            WHERE cc.category_id = $1
            ORDER BY c.created_at DESC
            LIMIT $2 OFFSET $3",
            category_id, limit, offset
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| Content {
            id: r.id,
            slug: r.slug,
            title: r.title,
            body: r.body,
            cover_image: r.cover_image,
            published: r.published,
            lang: r.lang,
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),  // 修复
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
        }).collect())
    }

    // ========== 公开方法（前台展示） ==========
    async fn list_by_category_slug(&self, slug: &str, limit: i64, offset: i64) -> Result<Vec<Content>> {
        let rows = sqlx::query!(
            "SELECT c.id, c.slug, c.title, c.body, c.published, c.cover_image, c.lang, c.translation_group, c.created_at, c.updated_at
             FROM contents c
             JOIN content_categories cc ON c.id = cc.content_id
             JOIN categories cat ON cc.category_id = cat.id
             WHERE cat.slug = $1 AND c.published = true
             ORDER BY c.created_at DESC
             LIMIT $2 OFFSET $3",
            slug, limit, offset
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| Content {
            id: r.id,
            slug: r.slug,
            title: r.title,
            body: r.body,
            published: r.published,
            cover_image: r.cover_image,
            lang: r.lang,
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),  // 修复
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
        }).collect())
    }

    async fn count_by_category_slug(&self, slug: &str) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count
             FROM contents c
             JOIN content_categories cc ON c.id = cc.content_id
             JOIN categories cat ON cc.category_id = cat.id
             WHERE cat.slug = $1 AND c.published = true",
            slug
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count.unwrap_or(0))
    }

    async fn get_content_by_slug_public(&self, slug: &str) -> Result<Option<Content>> {
        let row = sqlx::query!(
            "SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at
             FROM contents WHERE slug = $1 AND published = true",
            slug
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Content {
            id: r.id,
            slug: r.slug,
            title: r.title,
            body: r.body,
            published: r.published,
            cover_image: r.cover_image,
            lang: r.lang,
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),  // 修复
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
        }))
    }

    async fn get_category_by_slug(&self, slug: &str) -> Result<Option<Category>> {
        let row = sqlx::query!(
            "SELECT id, name, slug, description, parent_id, sort, display_type, show_in_nav, created_at, updated_at
             FROM categories WHERE slug = $1",
            slug
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Category {
            id: r.id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            parent_id: r.parent_id,
            sort: r.sort.unwrap_or(0),
            display_type: r.display_type,
            show_in_nav: r.show_in_nav.unwrap_or(true),
            created_at: r.created_at,
            updated_at: r.updated_at,
            children: None,
        }))
    }

    async fn update_categories_order(&self, updates: Vec<(i32, i32)>) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for (id, sort) in updates {
            sqlx::query!("UPDATE categories SET sort = $1 WHERE id = $2", sort, id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn list_all_published(&self) -> Result<Vec<SitemapEntry>> {
        let rows = sqlx::query!(
            r#"SELECT slug, updated_at FROM contents WHERE published = true ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

        Ok(rows
            .into_iter()
            .map(|r| SitemapEntry {
                slug: r.slug,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn get_all_public_category_slugs(&self) -> Result<Vec<String>> {
        let rows = sqlx::query!(r#"SELECT slug FROM categories"#)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(rows.into_iter().map(|r| r.slug).collect())
    }

    async fn get_related_contents(
        &self,
        content_id: Uuid,
        category_ids: &[i32],
        limit: i64,
    ) -> Result<Vec<Content>> {
        if category_ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query!(
            r#"SELECT DISTINCT c.id, c.slug, c.title, c.body, c.published, c.cover_image, c.lang, c.translation_group, c.created_at, c.updated_at
               FROM contents c
               JOIN content_categories cc ON c.id = cc.content_id
               WHERE cc.category_id = ANY($1)
                 AND c.id != $2
                 AND c.published = true
               ORDER BY c.created_at DESC
               LIMIT $3"#,
            category_ids,
            content_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Content {
            id: r.id,
            slug: r.slug,
            title: r.title,
            body: r.body,
            cover_image: r.cover_image,
            published: r.published,
            lang: r.lang,
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),  // 修复
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
        }).collect())
    }

    async fn list_by_category_slug_and_lang(&self, slug: &str, lang: &str, limit: i64, offset: i64) -> Result<Vec<Content>> {
        let rows = sqlx::query!(
            "SELECT c.id, c.slug, c.title, c.body, c.published, c.cover_image, c.lang, c.translation_group, c.created_at, c.updated_at
            FROM contents c
            JOIN content_categories cc ON c.id = cc.content_id
            JOIN categories cat ON cc.category_id = cat.id
            WHERE cat.slug = $1 AND c.published = true AND c.lang = $2
            ORDER BY c.created_at DESC
            LIMIT $3 OFFSET $4",
            slug, lang, limit, offset
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| Content {
            id: r.id,
            slug: r.slug,
            title: r.title,
            body: r.body,
            cover_image: r.cover_image,
            published: r.published,
            lang: r.lang,
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
        }).collect())
    }

    async fn count_by_category_slug_and_lang(&self, slug: &str, lang: &str) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count
            FROM contents c
            JOIN content_categories cc ON c.id = cc.content_id
            JOIN categories cat ON cc.category_id = cat.id
            WHERE cat.slug = $1 AND c.published = true AND c.lang = $2",
            slug, lang
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count.unwrap_or(0))
    }
    
    async fn search_published(&self, query: &str, limit: i64, offset: i64) -> Result<(Vec<Content>, i64)> {
        let like = format!("%{}%", query);
        let total_row = sqlx::query!(
            "SELECT COUNT(*) as count FROM contents WHERE published = true AND (title ILIKE $1 OR body ILIKE $1 OR slug ILIKE $1)",
            like
        )
        .fetch_one(&self.pool)
        .await?;
        let total = total_row.count.unwrap_or(0);

        let rows = sqlx::query!(
            r#"SELECT id, slug, title, body, published, cover_image, lang, translation_group, created_at, updated_at
            FROM contents
            WHERE published = true AND (title ILIKE $1 OR body ILIKE $1 OR slug ILIKE $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3"#,
            like, limit, offset
        )
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<Content> = rows.into_iter().map(|r| Content {
            id: r.id,
            slug: r.slug,
            title: r.title,
            body: r.body,
            cover_image: r.cover_image,
            published: r.published,
            lang: r.lang,
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
        }).collect();

        Ok((items, total))
    }
}

// 辅助函数：构建分类树
fn build_category_tree(categories: Vec<Category>, parent_id: Option<i32>) -> Vec<Category> {
    let mut result = Vec::new();
    for cat in categories.iter().filter(|c| c.parent_id == parent_id) {
        let mut node = cat.clone();
        node.children = Some(build_category_tree(categories.clone(), Some(node.id)));
        result.push(node);
    }
    result
}