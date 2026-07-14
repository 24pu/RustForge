use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::core::FavoriteRepository;
use crate::core::models::{Favorite, FavoriteWithContent, Content};

pub struct PostgresFavoriteRepo {
    pool: PgPool,
}

impl PostgresFavoriteRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_sort_order(sort_by: Option<&str>) -> &'static str {
        match sort_by {
            Some("created_at_asc") => "f.created_at ASC",
            Some("title_asc") => "c.title ASC NULLS LAST",
            Some("title_desc") => "c.title DESC NULLS LAST",
            _ => "f.created_at DESC",
        }
    }
}

#[async_trait::async_trait]
impl FavoriteRepository for PostgresFavoriteRepo {
    async fn create(&self, user_id: Uuid, content_id: Uuid, mark: Option<&str>) -> Result<Favorite> {
        let favorite = sqlx::query_as::<_, Favorite>(
            r#"
            INSERT INTO favorites (user_id, content_id, mark)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, content_id) DO UPDATE
            SET mark = EXCLUDED.mark, updated_at = now()
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(content_id)
        .bind(mark)
        .fetch_one(&self.pool)
        .await?;
        Ok(favorite)
    }

    async fn update_mark(&self, user_id: Uuid, content_id: Uuid, mark: Option<&str>) -> Result<Favorite> {
        let favorite = sqlx::query_as::<_, Favorite>(
            r#"
            UPDATE favorites
            SET mark = $3, updated_at = now()
            WHERE user_id = $1 AND content_id = $2
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(content_id)
        .bind(mark)
        .fetch_one(&self.pool)
        .await?;
        Ok(favorite)
    }

    async fn delete(&self, user_id: Uuid, content_id: Uuid) -> Result<()> {
        sqlx::query(
            "DELETE FROM favorites WHERE user_id = $1 AND content_id = $2"
        )
        .bind(user_id)
        .bind(content_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_user_and_content(&self, user_id: Uuid, content_id: Uuid) -> Result<Option<Favorite>> {
        let favorite = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE user_id = $1 AND content_id = $2"
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(favorite)
    }

    async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        mark_filter: Option<&str>,
        sort_by: Option<&str>,
    ) -> Result<Vec<FavoriteWithContent>> {
        // 构建 SQL 和参数
        let mut sql = String::from(
            r#"
            SELECT
                f.id,
                f.user_id,
                f.content_id,
                f.mark,
                f.created_at,
                f.updated_at,
                c.id,
                c.slug,
                c.title,
                c.body,
                c.published,
                c.cover_image,
                c.lang,
                c.translation_group,
                c.created_at,
                c.updated_at
            FROM favorites f
            INNER JOIN contents c ON c.id = f.content_id
            WHERE f.user_id = $1
            "#
        );

        let mut param_count = 2;
        if let Some(filter) = mark_filter {
            if filter == "no_mark" {
                sql.push_str(" AND (f.mark IS NULL OR f.mark = '')");
            } else {
                sql.push_str(&format!(" AND f.mark = ${}", param_count));
                param_count += 1;
            }
        }

        let order_by = Self::map_sort_order(sort_by);
        sql.push_str(&format!(" ORDER BY {}", order_by));

        sql.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        sql.push_str(&format!(" OFFSET ${}", param_count));

        // 构建查询
        let mut query = sqlx::query_as::<_, (i32, Uuid, Uuid, Option<String>, DateTime<Utc>, DateTime<Utc>, Uuid, String, String, String, bool, Option<String>, String, Option<Uuid>, DateTime<Utc>, DateTime<Utc>)>(&sql);

        query = query.bind(user_id);

        if let Some(filter) = mark_filter {
            if filter != "no_mark" {
                query = query.bind(filter);
            }
        }

        query = query.bind(limit).bind(offset);

        let rows = query.fetch_all(&self.pool).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let favorite = Favorite {
                id: row.0,
                user_id: row.1,
                content_id: row.2,
                mark: row.3,
                created_at: row.4,
                updated_at: row.5,
            };

            let content = Content {
                id: row.6,
                slug: row.7,
                title: row.8,
                body: row.9,
                published: row.10,
                cover_image: row.11,
                lang: row.12,
                translation_group: row.13.unwrap_or_default(),
                categories: vec![],
                created_at: row.14,
                updated_at: row.15,
            };

            result.push(FavoriteWithContent { favorite, content });
        }

        Ok(result)
    }

    async fn count_by_user(&self, user_id: Uuid) -> Result<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn count_by_user_filtered(
        &self,
        user_id: Uuid,
        mark_filter: Option<&str>,
    ) -> Result<i64> {
        let mut sql = String::from(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1"
        );
        let mut param_count = 2;
        if let Some(filter) = mark_filter {
            if filter == "no_mark" {
                sql.push_str(" AND (mark IS NULL OR mark = '')");
            } else {
                sql.push_str(&format!(" AND mark = ${}", param_count));
                param_count += 1;
            }
        }

        let mut query = sqlx::query_as::<_, (i64,)>(&sql);
        query = query.bind(user_id);
        if let Some(filter) = mark_filter {
            if filter != "no_mark" {
                query = query.bind(filter);
            }
        }
        let (count,) = query.fetch_one(&self.pool).await?;
        Ok(count)
    }
}