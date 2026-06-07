use sqlx::PgPool;
use serde_json::Value;
use anyhow::Result;
use crate::core::PluginSettingsRepository;

pub struct PostgresPluginSettingsRepo {
    pool: PgPool,
}

impl PostgresPluginSettingsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PluginSettingsRepository for PostgresPluginSettingsRepo {
    async fn get_settings(&self, plugin_name: &str) -> Result<Value> {
        let row = sqlx::query!(
            "SELECT settings FROM plugin_settings WHERE plugin_name = $1",
            plugin_name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(|r| r.settings)
            .unwrap_or(Value::Object(serde_json::Map::new())))
    }

    async fn save_settings(&self, plugin_name: &str, settings: Value) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO plugin_settings (plugin_name, settings) VALUES ($1, $2)
               ON CONFLICT (plugin_name) DO UPDATE SET settings = $2"#,
            plugin_name,
            settings as _
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}