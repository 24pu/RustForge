//core/models.rs

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

// 分类模型（支持树形结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // 以下字段用于前端树形展示（不存数据库）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Category>>,
    pub sort: i32,
    pub display_type: String,   // 'list', 'gallery', 'page'
    pub show_in_nav: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub cover_image: Option<String>,
    pub published: bool,
    pub lang: String,              // 新增
    pub translation_group: Uuid,   // 新增
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: i32,
    pub filename: String,          // 原始文件名
    pub storage_path: String,      // 存储的相对路径（例如 uploads/xxx.jpg）
    pub file_size: i64,
    pub mime_type: String,
    pub extension: String,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub folder_id: Option<i32>,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFolder {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MediaFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub file_path: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}



// ==================== 产品分类 ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategory {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub sort: i32,
    pub show_in_nav: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub children: Option<Vec<ProductCategory>>,  // 用于返回树形结构
     #[serde(skip_serializing_if = "Option::is_none")]
    pub product_count: Option<i64>,  // 添加产品数量字段
}

// ==================== 产品主表 ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub slug: String,
    pub lang: String,
     pub sku: Option<String>,
    pub translation_group: Uuid,

    // 基础信息
    pub name: String,
    pub dname: Option<String>,
    pub fullname: Option<String>,
    pub brand: Option<String>,
    pub cover_image: Option<String>,

    // 详细描述
    pub summary: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub points: Option<String>,
    pub dnote: Option<String>,

    // 尺寸相关
    pub csize: Option<String>,
    pub ussize: Option<String>,
    pub asize: Option<String>,
    pub fabric_type: Option<String>,

    // 价格与库存（原始文本，可包含范围）
    pub price: Option<String>,
    pub stock: Option<String>,

    // 包装与重量
    pub package: Option<String>,
    pub weight: Option<String>,

    // 发布状态
    pub published: bool,

    // 创建者
    pub user_id: Option<Uuid>,

    // 变体生成所需列表（逗号分隔或空格分隔）
    pub size_list: Option<String>,
    pub color_list: Option<String>,
    pub color_names: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // 关联数据（查询时填充，不直接映射数据库列）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub categories: Vec<ProductCategory>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub variants: Vec<ProductVariant>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub images: Vec<ProductImage>,
}

// ==================== 产品变体 ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub color: Option<String>,
    pub color_code: Option<String>,
    pub color_name: Option<String>,
    pub size: Option<String>,
    pub price: Option<f64>,
    pub stock: i32,
    pub weight: Option<String>,
    pub package_info: Option<String>,
    pub is_default: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== 产品图片 ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductImage {
    pub id: i32,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub url: String,
    pub name: Option<String>,
    pub original_name: Option<String>,
    pub color_code: Option<String>,
    pub image_type: String,       // "main", "swatch", "size_chart", "other"
    pub file_size: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmaTemplate {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub is_used: bool,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)] // 添加 FromRow
pub struct AttributeTemplate {
    pub id: i32,
    pub name: String,
    pub title: Option<String>,
    pub value: Option<String>,
    pub is_used: Option<bool>,    // 改为 Option<bool>
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttributeGroup {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Option<Uuid>,
    pub is_used: Option<bool>,    // 改为 Option<bool>
    pub sort_order: Option<i32>,  // 改为 Option<i32>
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


// 分组-模板关联（通常用于返回时附加排序）
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupTemplateRelation {
    pub group_id: i32,
    pub attribute_template_id: i32,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

// 产品属性值
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductAttributeValue {
    pub product_id: Uuid,
    pub attribute_template_id: i32,
    pub value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 扩展：分组详情（含模板列表）
#[derive(Debug, Serialize)]
pub struct GroupDetail {
    #[serde(flatten)]
    pub group: AttributeGroup,
    pub templates: Vec<AttributeTemplate>, // 已排序
}


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginHook {
    pub id: i32,
    pub plugin_name: String,
    pub hook_name: String,
    pub content: String,
    pub sort_order: Option<i32>,
    pub lang: Option<String>,
    pub enabled: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePluginHookRequest {
    pub plugin_name: String,
    pub hook_name: String,
    pub content: String,
    pub sort_order: Option<i32>,
    pub lang: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePluginHookRequest {
    pub content: Option<String>,
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
    pub lang: Option<String>,  // 新增
}
