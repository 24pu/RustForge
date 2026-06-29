// src/infrastructure/db/mod.rs
pub mod media_repo;
pub mod media_folder_repo;
pub mod user_repo;
pub mod content_repo;
pub mod plugin_repo;
pub mod plugin_settings_repo;
pub mod product_category_repo;
pub mod amatemplate_repo;
pub mod attribute_repo; // <-- 新增模块

pub mod product_repo;
pub mod plugin_hook_repo;

pub mod cart_repo;
pub mod order_repo;

pub use cart_repo::PostgresCartRepo;
pub use order_repo::PostgresOrderRepo;

pub use product_repo::PostgresProductRepo;

pub use plugin_repo::PostgresPluginRepo;
pub use plugin_hook_repo::PostgresPluginHookRepo;

pub use media_repo::PostgresMediaRepo;
pub use media_folder_repo::PostgresMediaFolderRepo;
use sqlx::postgres::PgPoolOptions;
use anyhow::Result;
pub use user_repo::PostgresUserRepo;
pub use content_repo::PostgresContentRepo;
pub use plugin_settings_repo::PostgresPluginSettingsRepo;
pub use amatemplate_repo::PostgresAmaTemplateRepo;
pub use attribute_repo::{
    PostgresAttributeTemplateRepo,
    PostgresAttributeGroupRepo,
    PostgresProductAttributeValueRepo,
}; // <-- 新增导出

pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}