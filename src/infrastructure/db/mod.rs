
pub mod media_repo;  // 新增
pub mod media_folder_repo;  // 新增
pub mod user_repo;
pub mod content_repo;
pub mod plugin_repo;
pub mod plugin_settings_repo;
pub use plugin_repo::PostgresPluginRepo;
pub use media_repo::PostgresMediaRepo;
pub use media_folder_repo::PostgresMediaFolderRepo;
use sqlx::postgres::PgPoolOptions;
use anyhow::Result;
pub use user_repo::PostgresUserRepo;
pub use content_repo::PostgresContentRepo;
pub use plugin_settings_repo::PostgresPluginSettingsRepo;



pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}