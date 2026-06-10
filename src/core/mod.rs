

pub mod models;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use crate::core::models::{User, Content, Permission, RoleInfo,Category,MediaFile,MediaFolder,Plugin};
use serde_json::Value;

// ---------- 主题相关 ----------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
}



#[async_trait]
pub trait Theme: Send + Sync {
    fn metadata(&self) -> &ThemeMetadata;
    async fn reload(&mut self) -> Result<(), ThemeError>;
    async fn render(&self, template_name: &str, context: HashMap<String, serde_json::Value>) -> Result<String, ThemeError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Render error: {0}")]
    RenderError(String),
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
    #[error("Failed to scan themes directory: {0}")]
    ScanError(String),
    #[error("Failed to load theme: {0}")]
    LoadError(String),
}



pub struct SitemapEntry {
    pub slug: String,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait ThemeManager: Send + Sync {
    fn list_themes(&self) -> Vec<ThemeMetadata>;
    fn active_theme(&self) -> String;  // 改为 String
    fn set_active_theme(&mut self, name: &str) -> Result<(), ThemeError>;
    async fn render(&self, template: &str, context: HashMap<String, serde_json::Value>) -> Result<String, ThemeError>;
    async fn reload_theme(&self, theme_name: &str) -> Result<(), ThemeError>;
}

// ---------- Repository traits ----------
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, email: &str, password_hash: &str, name: Option<&str>) -> Result<User>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn update_user(&self, id: Uuid, name: Option<String>) -> Result<User>;
   

    // 角色管理（使用字符串角色名）
    async fn list_roles(&self) -> Result<Vec<RoleInfo>>;
    async fn create_role(&self, name: &str, description: Option<&str>) -> Result<RoleInfo>;
    async fn update_role(&self, role_id: i32, name: &str, description: Option<&str>) -> Result<RoleInfo>;
    async fn delete_role(&self, role_id: i32) -> Result<bool>;

    // 用户角色关联
    async fn assign_role_by_name(&self, user_id: Uuid, role_name: &str) -> Result<()>;
    async fn revoke_role_by_name(&self, user_id: Uuid, role_name: &str) -> Result<()>;
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>>;
    async fn list_users_with_roles(&self, limit: i64, offset: i64) -> Result<Vec<(User, Vec<String>)>>;

    // 权限管理
    async fn list_permissions(&self) -> Result<Vec<Permission>>;
    async fn get_role_permissions(&self, role_id: i32) -> Result<Vec<Permission>>;
    async fn assign_permission(&self, role_id: i32, permission_id: i32) -> Result<()>;
    async fn revoke_permission(&self, role_id: i32, permission_id: i32) -> Result<()>;
    async fn update_role_permissions(&self, role_id: i32, permission_ids: &[i32]) -> Result<()>;
    async fn user_has_permission(&self, user_id: Uuid, permission: &str) -> Result<bool>;
    async fn delete_user(&self, id: Uuid) -> Result<bool>;
    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> Result<()>;
}

#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn create_content(
        &self, slug: &str, title: &str, body: &str, published: bool,
        cover_image: Option<String>, lang: &str, translation_group: Uuid,
    ) -> Result<Content>;
    async fn update_content(&self, id: Uuid, title: &str, body: &str, published: bool, cover_image: Option<String>) -> Result<Content>;
    async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>>;
    async fn list_published(&self, limit: i64) -> Result<Vec<Content>>;
    async fn get_content_by_id(&self, id: Uuid) -> Result<Option<Content>>;
    async fn delete_content(&self, id: Uuid) -> Result<bool>;
    async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn count_all(&self) -> Result<i64>;
    
    // 分类管理
    async fn list_categories_tree(&self, parent_id: Option<i32>) -> Result<Vec<Category>>;
    async fn get_category_by_id(&self, id: i32) -> Result<Option<Category>>;
    
    async fn delete_category(&self, id: i32) -> Result<bool>;

    // 内容-分类关联
    async fn get_content_categories(&self, content_id: Uuid) -> Result<Vec<Category>>;
    async fn set_content_categories(&self, content_id: Uuid, category_ids: &[i32]) -> Result<()>;

    async fn count_by_category(&self, category_id: i32) -> Result<i64>;
    async fn list_by_category(&self, category_id: i32, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn update_categories_order(&self, updates: Vec<(i32, i32)>) -> Result<()>;

    async fn list_by_category_slug(&self, slug: &str, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn count_by_category_slug(&self, slug: &str) -> Result<i64>;
    async fn get_content_by_slug_public(&self, slug: &str) -> Result<Option<Content>>;
    async fn get_category_by_slug(&self, slug: &str) -> Result<Option<Category>>;
    async fn list_all_published(&self) -> Result<Vec<SitemapEntry>>;
    async fn get_all_public_category_slugs(&self) -> Result<Vec<String>>;

    async fn create_category(&self, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>, display_type: &str, show_in_nav: bool) -> Result<Category>;
    async fn update_category(&self, id: i32, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>, display_type: &str, show_in_nav: bool) -> Result<Category>;
    /// 获取相关文章：基于分类，排除当前文章，已发布，按创建时间倒序
    async fn get_related_contents(
        &self,
        content_id: Uuid,
        category_ids: &[i32],
        limit: i64,
    ) -> Result<Vec<Content>>;
      // 新增方法
    async fn get_content_by_slug_and_lang(&self, slug: &str, lang: &str) -> Result<Option<Content>>;
    async fn get_content_translations(&self, translation_group: Uuid, exclude_lang: &str) -> Result<Vec<Content>>;
    async fn list_by_category_slug_and_lang(&self, slug: &str, lang: &str, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn count_by_category_slug_and_lang(&self, slug: &str, lang: &str) -> Result<i64>;
    async fn search_published(&self, query: &str, limit: i64, offset: i64) -> Result<(Vec<Content>, i64)>;

}
#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn create_media(&self, media: &MediaFile) -> Result<MediaFile>;
    async fn list_media(&self, limit: i64, offset: i64) -> Result<Vec<MediaFile>>;
    async fn count_media(&self) -> Result<i64>;
    async fn get_media_by_id(&self, id: i32) -> Result<Option<MediaFile>>;
    async fn delete_media(&self, id: i32) -> Result<bool>;
    // 在 MediaRepository trait 中添加
    async fn list_media_by_folder(&self, folder_id: Option<i32>, limit: i64, offset: i64) -> Result<Vec<MediaFile>>;
    async fn count_media_by_folder(&self, folder_id: Option<i32>) -> Result<i64>;
 }
#[async_trait]
pub trait MediaFolderRepository: Send + Sync {
    async fn list_folders_tree(&self, parent_id: Option<i32>) -> Result<Vec<MediaFolder>>;
    async fn create_folder(&self, name: &str, parent_id: Option<i32>, created_by: Option<Uuid>) -> Result<MediaFolder>;
    async fn update_folder(&self, id: i32, name: &str) -> Result<MediaFolder>;
    async fn delete_folder(&self, id: i32) -> Result<bool>;
    async fn get_folder_by_id(&self, id: i32) -> Result<Option<MediaFolder>>;
}





#[async_trait]
pub trait PluginRepository: Send + Sync {
    async fn list_plugins(&self) -> Result<Vec<Plugin>>;
    async fn get_plugin_by_id(&self, id: i32) -> Result<Option<Plugin>>;
    async fn get_plugin_by_name(&self, name: &str) -> Result<Option<Plugin>>;
    async fn create_plugin(&self, plugin: &Plugin) -> Result<Plugin>;
    async fn update_plugin(&self, id: i32, enabled: bool) -> Result<()>;
    async fn delete_plugin(&self, id: i32) -> Result<bool>;
}



#[async_trait::async_trait]
pub trait PluginSettingsRepository: Send + Sync {
    async fn get_settings(&self, plugin_name: &str) -> Result<Value>;
    async fn save_settings(&self, plugin_name: &str, settings: Value) -> Result<()>;
}