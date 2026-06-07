use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct CreateContentRequest {
    pub slug: String,
    pub title: String,
    pub body: String,
    pub published: Option<bool>,
    pub category_ids: Option<Vec<i32>>,
    pub cover_image: Option<String>,
    pub lang: Option<String>,
    pub translation_group: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateContentRequest {
    pub title: String,
    pub body: String,
    pub published: Option<bool>,
    pub category_ids: Option<Vec<i32>>,
    pub cover_image: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRolesRequest {
    pub roles: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub permission_ids: Vec<i32>,
}

#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    pub site_name: String,
    pub default_per_page: i32,
    pub theme_color: String,
    pub seo_title: String,
    pub seo_description: String,
    pub seo_keywords: String,
    pub logo_url: String,
    pub favicon_url: String,
    pub allowed_file_types: String,
    pub max_file_size_mb: i32,
}

#[derive(Deserialize)]
pub struct ListContentsParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub category_id: Option<i32>,
    pub keyword: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub display_type: Option<String>,
    pub show_in_nav: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub display_type: Option<String>,
    pub show_in_nav: Option<bool>,
}

#[derive(Deserialize)]
pub struct ReorderCategoriesRequest {
    pub items: Vec<ReorderItem>,
}

#[derive(Deserialize)]
pub struct ReorderItem {
    pub id: i32,
    pub sort: i32,
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct RenameFolderRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ListMediaParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub folder_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct CategoryPageParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct UserInfo {
    pub is_logged_in: bool,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
}

impl UserInfo {
    pub fn anonymous() -> Self {
        UserInfo {
            is_logged_in: false,
            user_id: None,
            user_name: None,
        }
    }
}

#[derive(Deserialize)]
pub struct InstallPluginRequest {
    pub file_path: Option<String>,
}