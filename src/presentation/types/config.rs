// src/presentation/types/config.rs

use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
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
    // 产品设置
    pub product_allowed_image_types: Option<String>,
    pub product_max_image_size_mb: Option<f64>,
    pub product_max_images_count: Option<i32>,
    pub product_auto_thumbnail: Option<bool>,
    pub product_thumbnail_width: Option<i32>,
    pub product_thumbnail_height: Option<i32>,
    pub product_size_inch: bool,  // 改为 bool，不是 Option<bool>
    pub site_url: Option<String>,   // 新增
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
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
    // 产品设置
    pub product_allowed_image_types: String,
    pub product_max_image_size_mb: f64,
    pub product_max_images_count: i32,
    pub product_auto_thumbnail: bool,
    pub product_thumbnail_width: i32,
    pub product_thumbnail_height: i32,
    pub product_size_inch: bool,  // 改为 bool，不是 Option<bool>
    pub site_url: String,  
}