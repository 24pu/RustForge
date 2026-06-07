use sqlx::PgPool;
use anyhow::Result;
use async_trait::async_trait;
use crate::core::models::Plugin;
use crate::core::PluginRepository;
use chrono::{DateTime, Utc};





pub struct PostgresPluginRepo {
    pool: PgPool,
}

impl PostgresPluginRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PluginRepository for PostgresPluginRepo {
    async fn list_plugins(&self) -> Result<Vec<Plugin>> {
        let rows = sqlx::query!(
            "SELECT id, name, version, author, description, file_path, enabled, created_at, updated_at FROM plugins ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| Plugin {
            id: r.id,
            name: r.name,
            version: r.version,
            author: r.author,
            description: r.description,
            file_path: r.file_path,
            enabled: r.enabled,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect())
    }

    async fn get_plugin_by_id(&self, id: i32) -> Result<Option<Plugin>> {
        let row = sqlx::query!(
            "SELECT id, name, version, author, description, file_path, enabled, created_at, updated_at FROM plugins WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Plugin {
            id: r.id,
            name: r.name,
            version: r.version,
            author: r.author,
            description: r.description,
            file_path: r.file_path,
            enabled: r.enabled,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn get_plugin_by_name(&self, name: &str) -> Result<Option<Plugin>> {
        let row = sqlx::query!(
            "SELECT id, name, version, author, description, file_path, enabled, created_at, updated_at FROM plugins WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Plugin {
            id: r.id,
            name: r.name,
            version: r.version,
            author: r.author,
            description: r.description,
            file_path: r.file_path,
            enabled: r.enabled,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn create_plugin(&self, plugin: &Plugin) -> Result<Plugin> {
        let row = sqlx::query!(
            "INSERT INTO plugins (name, version, author, description, file_path, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, version, author, description, file_path, enabled, created_at, updated_at",
            plugin.name, plugin.version, plugin.author, plugin.description, plugin.file_path, plugin.enabled
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(Plugin {
            id: row.id,
            name: row.name,
            version: row.version,
            author: row.author,
            description: row.description,
            file_path: row.file_path,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn update_plugin(&self, id: i32, enabled: bool) -> Result<()> {
        sqlx::query!("UPDATE plugins SET enabled = $1, updated_at = now() WHERE id = $2", enabled, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_plugin(&self, id: i32) -> Result<bool> {
        // 先获取文件路径以便删除文件
        let plugin = self.get_plugin_by_id(id).await?;
        if let Some(p) = plugin {
            // 删除文件（不要求成功，文件可能不存在）
            let _ = tokio::fs::remove_file(&p.file_path).await;
        }
        let res = sqlx::query!("DELETE FROM plugins WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}