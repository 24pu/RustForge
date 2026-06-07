use sqlx::{PgPool, Row};
use anyhow::Result;
use async_trait::async_trait;
use crate::core::models::MediaFile;
use crate::core::MediaRepository;

pub struct PostgresMediaRepo {
    pool: PgPool,
}

impl PostgresMediaRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MediaRepository for PostgresMediaRepo {
    async fn create_media(&self, media: &MediaFile) -> Result<MediaFile> {
        let row = sqlx::query(
            "INSERT INTO media_files (filename, storage_path, file_size, mime_type, extension, uploaded_by, folder_id, thumbnail_path)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, filename, storage_path, file_size, mime_type, extension, uploaded_by, folder_id, thumbnail_path, created_at, updated_at"
        )
        .bind(&media.filename)
        .bind(&media.storage_path)
        .bind(media.file_size)
        .bind(&media.mime_type)
        .bind(&media.extension)
        .bind(media.uploaded_by)
        .bind(media.folder_id)
        .bind(&media.thumbnail_path)
        .fetch_one(&self.pool)
        .await?;
        Ok(MediaFile {
            id: row.get("id"),
            filename: row.get("filename"),
            storage_path: row.get("storage_path"),
            file_size: row.get("file_size"),
            mime_type: row.get("mime_type"),
            extension: row.get("extension"),
            uploaded_by: row.get("uploaded_by"),
            folder_id: row.get("folder_id"),
            thumbnail_path: row.get("thumbnail_path"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_media(&self, limit: i64, offset: i64) -> Result<Vec<MediaFile>> {
        let rows = sqlx::query(
            "SELECT id, filename, storage_path, file_size, mime_type, extension, uploaded_by, folder_id,thumbnail_path, created_at, updated_at
             FROM media_files
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| MediaFile {
            id: r.get("id"),
            filename: r.get("filename"),
            storage_path: r.get("storage_path"),
            file_size: r.get("file_size"),
            mime_type: r.get("mime_type"),
            extension: r.get("extension"),
            uploaded_by: r.get("uploaded_by"),
            folder_id: r.get("folder_id"),
            thumbnail_path: r.get("thumbnail_path"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn count_media(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM media_files")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("count"))
    }

    async fn get_media_by_id(&self, id: i32) -> Result<Option<MediaFile>> {
        let row = sqlx::query(
            "SELECT id, filename, storage_path, file_size, mime_type, extension, uploaded_by, folder_id,thumbnail_path, created_at, updated_at
             FROM media_files WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| MediaFile {
            id: r.get("id"),
            filename: r.get("filename"),
            storage_path: r.get("storage_path"),
            file_size: r.get("file_size"),
            mime_type: r.get("mime_type"),
            extension: r.get("extension"),
            uploaded_by: r.get("uploaded_by"),
            folder_id: r.get("folder_id"),
            thumbnail_path: r.get("thumbnail_path"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn delete_media(&self, id: i32) -> Result<bool> {
        let res = sqlx::query("DELETE FROM media_files WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    async fn list_media_by_folder(&self, folder_id: Option<i32>, limit: i64, offset: i64) -> Result<Vec<MediaFile>> {
        let mut query = String::from(
            "SELECT id, filename, storage_path, file_size, mime_type, extension, uploaded_by, folder_id,thumbnail_path, created_at, updated_at
             FROM media_files"
        );
        let rows = if let Some(fid) = folder_id {
            query.push_str(" WHERE folder_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3");
            sqlx::query(&query)
                .bind(fid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            query.push_str(" WHERE folder_id IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2");
            sqlx::query(&query)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };
        Ok(rows.iter().map(|r| MediaFile {
            id: r.get("id"),
            filename: r.get("filename"),
            storage_path: r.get("storage_path"),
            file_size: r.get("file_size"),
            mime_type: r.get("mime_type"),
            extension: r.get("extension"),
            uploaded_by: r.get("uploaded_by"),
            folder_id: r.get("folder_id"),
            thumbnail_path: r.get("thumbnail_path"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn count_media_by_folder(&self, folder_id: Option<i32>) -> Result<i64> {
        let row = if let Some(fid) = folder_id {
            sqlx::query("SELECT COUNT(*) as count FROM media_files WHERE folder_id = $1")
                .bind(fid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT COUNT(*) as count FROM media_files WHERE folder_id IS NULL")
                .fetch_one(&self.pool)
                .await?
        };
        let count: i64 = row.get("count");
        Ok(count)
    }
}