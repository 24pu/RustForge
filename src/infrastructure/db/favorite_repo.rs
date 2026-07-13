// src/infrastructure/db/favorite_repo.rs

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

    async fn list_by_user(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<FavoriteWithContent>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                f.id as "favorite_id",
                f.user_id as "favorite_user_id",
                f.content_id as "favorite_content_id",
                f.mark as "favorite_mark",
                f.created_at as "favorite_created_at",
                f.updated_at as "favorite_updated_at",
                c.id as "content_id",
                c.slug,
                c.title,
                c.body,
                c.published,
                c.cover_image,
                c.lang,
                c.translation_group,
                c.created_at as "content_created_at",
                c.updated_at as "content_updated_at"
            FROM favorites f
            INNER JOIN contents c ON c.id = f.content_id
            WHERE f.user_id = $1
            ORDER BY f.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            // favorites 表的 created_at 和 updated_at 是 Option（可能因未显式 NOT NULL）
            // 但因有默认值，安全解包
            let favorite = Favorite {
                id: row.favorite_id,
                user_id: row.favorite_user_id,
                content_id: row.favorite_content_id,
                mark: row.favorite_mark,
                created_at: row.favorite_created_at.unwrap(),
                updated_at: row.favorite_updated_at.unwrap(),
            };

            // contents 表的 created_at 和 updated_at 是非 Option (已显式 NOT NULL)
            // translation_group 可能为 NULL，使用默认值
            let content = Content {
                id: row.content_id,
                slug: row.slug,
                title: row.title,
                body: row.body,
                published: row.published,
                cover_image: row.cover_image,
                lang: row.lang,
                translation_group: row.translation_group.unwrap_or_default(),
                categories: vec![], // 分类未加载，可后续优化
                created_at: row.content_created_at,       // 直接赋值
                updated_at: row.content_updated_at,       // 直接赋值
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
}