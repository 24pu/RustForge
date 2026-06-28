// core.rs

pub mod models;

use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use serde_json::Value;

use crate::core::models::{User, Content, Permission, RoleInfo, Category, MediaFile, MediaFolder, Plugin};
use crate::core::models::{Product, ProductVariant, ProductImage, ProductCategory,AmaTemplate};
use crate::core::models::{AttributeTemplate, AttributeGroup, GroupTemplateRelation, ProductAttributeValue,GroupDetail};
use crate::core::models::{PluginHook,CreatePluginHookRequest,UpdatePluginHookRequest};


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
    fn active_theme(&self) -> String;
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
    async fn delete_user(&self, id: Uuid) -> Result<bool>;
    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> Result<()>;

    // 角色管理
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
}

#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn create_content(
        &self, slug: &str, title: &str, body: &str, published: bool,
        cover_image: Option<String>, lang: &str, translation_group: Uuid,
    ) -> Result<Content>;
    async fn update_content(&self, id: Uuid, title: &str, body: &str, published: bool, cover_image: Option<String>) -> Result<Content>;
    async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>>;
    async fn get_content_by_slug_and_lang(&self, slug: &str, lang: &str) -> Result<Option<Content>>;
    async fn get_content_translations(&self, translation_group: Uuid, exclude_lang: &str) -> Result<Vec<Content>>;
    async fn list_published(&self, limit: i64) -> Result<Vec<Content>>;
    async fn get_content_by_id(&self, id: Uuid) -> Result<Option<Content>>;
    async fn delete_content(&self, id: Uuid) -> Result<bool>;
    async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn count_all(&self) -> Result<i64>;
    async fn list_all_published(&self) -> Result<Vec<SitemapEntry>>;
    
    // 分类管理
    async fn list_categories_tree(&self, parent_id: Option<i32>) -> Result<Vec<Category>>;
    async fn get_category_by_id(&self, id: i32) -> Result<Option<Category>>;
    async fn get_category_by_slug(&self, slug: &str) -> Result<Option<Category>>;
    async fn create_category(&self, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>, display_type: &str, show_in_nav: bool) -> Result<Category>;
    async fn update_category(&self, id: i32, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>, display_type: &str, show_in_nav: bool) -> Result<Category>;
    async fn delete_category(&self, id: i32) -> Result<bool>;
    async fn update_categories_order(&self, updates: Vec<(i32, i32)>) -> Result<()>;
    async fn get_all_public_category_slugs(&self) -> Result<Vec<String>>;

    // 内容-分类关联
    async fn get_content_categories(&self, content_id: Uuid) -> Result<Vec<Category>>;
    async fn set_content_categories(&self, content_id: Uuid, category_ids: &[i32]) -> Result<()>;
    async fn count_by_category(&self, category_id: i32) -> Result<i64>;
    async fn list_by_category(&self, category_id: i32, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn list_by_category_slug(&self, slug: &str, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn count_by_category_slug(&self, slug: &str) -> Result<i64>;
    async fn list_by_category_slug_and_lang(&self, slug: &str, lang: &str, limit: i64, offset: i64) -> Result<Vec<Content>>;
    async fn count_by_category_slug_and_lang(&self, slug: &str, lang: &str) -> Result<i64>;
    async fn get_content_by_slug_public(&self, slug: &str) -> Result<Option<Content>>;
    
    // 搜索
    async fn search_published(&self, query: &str, lang: &str, limit: i64, offset: i64) -> Result<(Vec<Content>, i64)>;
    
    // 相关文章
    async fn get_related_contents(&self, content_id: Uuid, category_ids: &[i32], limit: i64) -> Result<Vec<Content>>;
}

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn create_media(&self, media: &MediaFile) -> Result<MediaFile>;
    async fn list_media(&self, limit: i64, offset: i64) -> Result<Vec<MediaFile>>;
    async fn count_media(&self) -> Result<i64>;
    async fn get_media_by_id(&self, id: i32) -> Result<Option<MediaFile>>;
    async fn delete_media(&self, id: i32) -> Result<bool>;
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

#[async_trait]
pub trait PluginSettingsRepository: Send + Sync {
    async fn get_settings(&self, plugin_name: &str) -> Result<Value>;
    async fn save_settings(&self, plugin_name: &str, settings: Value) -> Result<()>;
}

// ---------- 产品相关 ----------

#[async_trait]
pub trait ProductCategoryRepository: Send + Sync {
    async fn create_category(&self, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>) -> Result<ProductCategory>;
    async fn update_category(&self, id: i32, name: &str, slug: &str, description: Option<&str>, parent_id: Option<i32>) -> Result<ProductCategory>;
    async fn delete_category(&self, id: i32) -> Result<bool>;
    async fn list_categories_tree(&self, parent_id: Option<i32>) -> Result<Vec<ProductCategory>>;
    async fn get_category_by_id(&self, id: i32) -> Result<Option<ProductCategory>>;
    async fn get_category_by_slug(&self, slug: &str) -> Result<Option<ProductCategory>>;
}

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn create_product(&self, input: CreateProductInput) -> Result<Product>;
    async fn update_product(&self, id: Uuid, input: UpdateProductInput) -> Result<Product>;
    async fn delete_product(&self, id: Uuid) -> Result<bool>;
    async fn get_product_by_id(&self, id: Uuid) -> Result<Option<Product>>;
    async fn get_product_by_slug(&self, slug: &str) -> Result<Option<Product>>;
    async fn list_products(&self, pagination: Pagination, filters: ProductFilters) -> Result<(Vec<Product>, i64)>;
    
    // 变体
    async fn create_variant(&self, input: CreateVariantInput) -> Result<ProductVariant>;
    async fn update_variant(&self, id: Uuid, input: UpdateVariantInput) -> Result<ProductVariant>;
    async fn delete_variant(&self, id: Uuid) -> Result<bool>;
    async fn list_variants(&self, product_id: Uuid) -> Result<Vec<ProductVariant>>;
    
    // 图片
    async fn add_product_image(&self, input: AddProductImageInput) -> Result<ProductImage>;
    async fn delete_product_image(&self, id: i32) -> Result<bool>;
    async fn get_product_images(&self, product_id: Uuid, variant_id: Option<Uuid>) -> Result<Vec<ProductImage>>;
    
    // 分类关联
    async fn set_product_categories(&self, product_id: Uuid, category_ids: &[i32]) -> Result<()>;
    async fn get_product_categories(&self, product_id: Uuid) -> Result<Vec<ProductCategory>>;

    // 分类相关方法
   
    // 批量生成变体
    async fn generate_variants(&self, product_id: Uuid, colors: &[ColorWithName], sizes: &[String], default_price: Option<f64>) -> Result<Vec<ProductVariant>>;
}

// ---------- 产品输入类型 ----------

#[derive(Debug, Clone)]
pub struct ColorWithName {
    pub code: String,
    pub name: String,
}

#[derive(Debug)]
pub struct CreateProductInput {
    pub slug: String,
    pub lang: Option<String>,
    pub name: String,
    pub sku: String,
    pub dname: Option<String>,
    pub fullname: Option<String>,
    pub brand: Option<String>,
    pub cover_image: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub points: Option<String>,
    pub dnote: Option<String>,
    pub csize: Option<String>,
    pub ussize: Option<String>,
    pub asize: Option<String>,
    pub fabric_type: Option<String>,
    pub price: Option<String>,
    pub stock: Option<String>,
    pub package: Option<String>,
    pub weight: Option<String>,
    pub published: Option<bool>,
    pub translation_group: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub size_list: Option<String>,
    pub color_list: Option<String>,
    pub color_names: Option<String>,
}

impl Default for CreateProductInput {
    fn default() -> Self {
        Self {
            slug: String::new(),
            lang: None,
            sku: String::new(),
            name: String::new(),
            dname: None,
            fullname: None,
            brand: None,
            cover_image: None,
            summary: None,
            description: None,
            keywords: None,
            points: None,
            dnote: None,
            csize: None,
            ussize: None,
            asize: None,
            fabric_type: None,
            price: None,
            stock: None,
            package: None,
            weight: None,
            published: Some(false),
            translation_group: None,
            user_id: None,
            size_list: None,
            color_list: None,
            color_names: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct UpdateProductInput {
    pub name: Option<String>,
    pub sku: Option<String>,
    pub dname: Option<String>,
    pub fullname: Option<String>,
    pub brand: Option<String>,
    pub cover_image: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub points: Option<String>,
    pub dnote: Option<String>,
    pub csize: Option<String>,
    pub ussize: Option<String>,
    pub asize: Option<String>,
    pub fabric_type: Option<String>,
    pub price: Option<String>,
    pub stock: Option<String>,
    pub package: Option<String>,
    pub weight: Option<String>,
    pub published: Option<bool>,
    pub translation_group: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub size_list: Option<String>,
    pub color_list: Option<String>,
    pub color_names: Option<String>,
}

#[derive(Debug)]
pub struct CreateVariantInput {
    pub product_id: Uuid,
    pub sku: String,
    pub color: Option<String>,
    pub color_code: Option<String>,
    pub color_name: Option<String>,
    pub size: Option<String>,
    pub price: Option<f64>,
    pub stock: i32,
}

#[derive(Debug, Default)]
pub struct UpdateVariantInput {
    pub price: Option<f64>,
    pub stock: Option<i32>,
    pub is_default: Option<bool>,
}

#[derive(Debug)]
pub struct AddProductImageInput {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub url: String,
    pub original_name: Option<String>,
    pub color_code: Option<String>,
    pub image_type: String,
    pub file_size: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

// ---------- 分页和过滤 ----------

#[derive(Debug)]
pub struct Pagination {
    pub page: usize,
    pub per_page: usize,
}

impl Pagination {
    pub fn new(page: usize, per_page: usize) -> Self {
        Self { page, per_page }
    }
    
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.per_page
    }
}

#[derive(Debug, Default)]
pub struct ProductFilters {
    pub category_id: Option<i32>,
    pub keyword: Option<String>,
    pub published: Option<bool>,
}

// ---------- 辅助函数 ----------
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}



#[async_trait]
pub trait AmaTemplateRepository: Send + Sync {
    async fn list(&self, user_id: Uuid) -> Result<Vec<AmaTemplate>>;
    async fn get_by_id(&self, id: i32, user_id: Uuid) -> Result<Option<AmaTemplate>>;
    async fn create(&self, name: &str, value: &str, is_used: bool, user_id: Uuid) -> Result<AmaTemplate>;
    async fn update(&self, id: i32, name: &str, value: &str, is_used: bool, user_id: Uuid) -> Result<AmaTemplate>;
    async fn delete(&self, id: i32, user_id: Uuid) -> Result<bool>;
}



// ---------- 属性模板 ----------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateAttributeTemplateInput {
    pub name: String,
    pub title: Option<String>,
    pub value: Option<String>,
    pub is_used: Option<bool>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateAttributeTemplateInput {
    pub name: Option<String>,
    pub title: Option<String>,
    pub value: Option<String>,
    pub is_used: Option<bool>,
}

#[async_trait]
pub trait AttributeTemplateRepository: Send + Sync {
    async fn list_templates(&self, is_used: Option<bool>) -> Result<Vec<AttributeTemplate>>;
    async fn get_template_by_id(&self, id: i32) -> Result<Option<AttributeTemplate>>;
    async fn create_template(&self, input: CreateAttributeTemplateInput) -> Result<AttributeTemplate>;
    async fn update_template(&self, id: i32, input: UpdateAttributeTemplateInput) -> Result<AttributeTemplate>;
    async fn delete_template(&self, id: i32) -> Result<bool>;
}

// ---------- 属性分组 ----------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateAttributeGroupInput {
    pub name: String,
    pub description: Option<String>,
    pub user_id: Option<Uuid>,
    pub is_used: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateAttributeGroupInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_used: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddTemplateToGroupInput {
    pub attribute_template_id: i32,
    pub sort_order: Option<i32>,
}

#[async_trait]
pub trait AttributeGroupRepository: Send + Sync {
    // 分组 CRUD
    async fn list_groups(&self, user_id: Option<Uuid>) -> Result<Vec<AttributeGroup>>;
    async fn get_group_by_id(&self, id: i32) -> Result<Option<AttributeGroup>>;
    async fn create_group(&self, input: CreateAttributeGroupInput) -> Result<AttributeGroup>;
    async fn update_group(&self, id: i32, input: UpdateAttributeGroupInput) -> Result<AttributeGroup>;
    async fn delete_group(&self, id: i32) -> Result<bool>;

    // 分组-模板关联
    async fn get_group_templates(&self, group_id: i32) -> Result<Vec<AttributeTemplate>>;
    async fn add_template_to_group(&self, group_id: i32, template_id: i32, sort_order: i32) -> Result<()>;
    async fn remove_template_from_group(&self, group_id: i32, template_id: i32) -> Result<()>;
    async fn update_template_sort_in_group(&self, group_id: i32, template_id: i32, sort_order: i32) -> Result<()>;
}

// ---------- 产品属性值 ----------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductAttributeValueInput {
    pub attribute_template_id: i32,
    pub value: Option<String>,
}

#[async_trait]
pub trait ProductAttributeValueRepository: Send + Sync {
    async fn get_product_attribute_values(&self, product_id: Uuid) -> Result<Vec<ProductAttributeValue>>;
    async fn set_product_attribute_values(&self, product_id: Uuid, values: &[ProductAttributeValueInput]) -> Result<()>;
    async fn delete_product_attribute_value(&self, product_id: Uuid, attribute_template_id: i32) -> Result<bool>;
}

// ---------- 扩展产品仓库，支持分组 ----------
// 如果你不想增加新 trait，也可以直接在 ProductRepository 中添加：
// async fn set_product_attribute_group(&self, product_id: Uuid, group_id: Option<i32>) -> Result<()>;
// 但这里建议在具体实现中直接调用 product 表的更新字段，不需要定义新 trait，因为已有 update_product。

#[async_trait]
pub trait PluginHookRepository: Send + Sync {
    async fn list_by_hook(
        &self,
        hook_name: &str,
        lang: &str,
        enabled: Option<bool>,
    ) -> Result<Vec<PluginHook>>;
    async fn get_by_id(&self, id: i32) -> Result<Option<PluginHook>>;
    async fn create(&self, req: &CreatePluginHookRequest) -> Result<PluginHook>;
    async fn update(&self, id: i32, req: &UpdatePluginHookRequest) -> Result<PluginHook>;
    async fn delete(&self, id: i32) -> Result<bool>;
    // 新增：查询所有通用钩子（lang=''）
    async fn list_all_generic(&self, enabled: Option<bool>) -> Result<Vec<PluginHook>>;
    async fn list_by_plugin(&self, plugin_name: &str, enabled: Option<bool>) -> Result<Vec<PluginHook>>;
    async fn list_by_lang(&self, lang: &str, enabled: Option<bool>) -> Result<Vec<PluginHook>>;
    /// 查询指定语言且所属插件已启用的钩子（用于前台渲染）
    async fn list_enabled_by_lang(&self, lang: &str) -> Result<Vec<PluginHook>>;
}