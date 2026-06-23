// types/mod.rs

mod auth;
mod content;
mod category;
mod media;
mod role;
mod config;
mod plugin;
mod product;
mod product_category;
mod common;
pub mod amatemplate;
pub use amatemplate::*;
// 认证相关
pub use auth::{RegisterRequest, LoginRequest, LoginResponse, UserInfo};

// 内容管理
pub use content::{CreateContentRequest, UpdateContentRequest, ListContentsParams};

// 分类管理
pub use category::{CreateCategoryRequest, UpdateCategoryRequest, ReorderCategoriesRequest, ReorderItem, CategoryPageParams};

// 媒体库
pub use media::{CreateFolderRequest, RenameFolderRequest, ListMediaParams};

// 角色权限
pub use role::{UpdateRolesRequest, CreateRoleRequest, UpdateRoleRequest, UpdateRolePermissionsRequest};



// 插件管理
pub use plugin::InstallPluginRequest;

// 产品管理
pub use product::{CreateProductRequest, UpdateProductRequest, ProductResponse, GenerateVariantsRequest, ColorInfo};

// 产品分类
pub use product_category::{CreateProductCategoryRequest, UpdateProductCategoryRequest};

// 通用类型
pub use common::{ApiResponse, PaginatedResponse};
// 系统配置
pub use config::{UpdateConfigRequest, ConfigResponse};  // 确保这一行存在