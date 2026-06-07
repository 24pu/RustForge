use sqlx::PgPool;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::core::models::MediaFolder;
use crate::core::MediaFolderRepository;

pub struct PostgresMediaFolderRepo {
    pool: PgPool,
}

impl PostgresMediaFolderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MediaFolderRepository for PostgresMediaFolderRepo {
    async fn list_folders_tree(&self, parent_id: Option<i32>) -> Result<Vec<MediaFolder>> {
        let rows = sqlx::query!(
            "SELECT id, name, parent_id, created_by, created_at, updated_at FROM media_folders ORDER BY parent_id NULLS FIRST, name"
        )
        .fetch_all(&self.pool)
        .await?;
        let all: Vec<MediaFolder> = rows.into_iter().map(|r| MediaFolder {
            id: r.id,
            name: r.name,
            parent_id: r.parent_id,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
            children: None,
        }).collect();
        Ok(build_folder_tree(all, parent_id))
    }

    async fn create_folder(&self, name: &str, parent_id: Option<i32>, created_by: Option<Uuid>) -> Result<MediaFolder> {
        let row = sqlx::query!(
            "INSERT INTO media_folders (name, parent_id, created_by) VALUES ($1, $2, $3)
             RETURNING id, name, parent_id, created_by, created_at, updated_at",
            name, parent_id, created_by
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(MediaFolder {
            id: row.id,
            name: row.name,
            parent_id: row.parent_id,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
            children: None,
        })
    }

    async fn update_folder(&self, id: i32, name: &str) -> Result<MediaFolder> {
        let row = sqlx::query!(
            "UPDATE media_folders SET name = $1, updated_at = now() WHERE id = $2
             RETURNING id, name, parent_id, created_by, created_at, updated_at",
            name, id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(MediaFolder {
            id: row.id,
            name: row.name,
            parent_id: row.parent_id,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
            children: None,
        })
    }

    async fn delete_folder(&self, id: i32) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM media_folders WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    async fn get_folder_by_id(&self, id: i32) -> Result<Option<MediaFolder>> {
        let row = sqlx::query!(
            "SELECT id, name, parent_id, created_by, created_at, updated_at FROM media_folders WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| MediaFolder {
            id: r.id,
            name: r.name,
            parent_id: r.parent_id,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
            children: None,
        }))
    }
}

fn build_folder_tree(folders: Vec<MediaFolder>, parent_id: Option<i32>) -> Vec<MediaFolder> {
    let mut result = Vec::new();
    for folder in folders.iter().filter(|f| f.parent_id == parent_id) {
        let mut node = folder.clone();
        node.children = Some(build_folder_tree(folders.clone(), Some(node.id)));
        result.push(node);
    }
    result
}