// src/infrastructure/color_code.rs

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ColorConfig {
    pub color_map: HashMap<String, String>,
    pub color_name: HashMap<String, String>,
}

// 静态加载颜色配置
static COLOR_CONFIG: Lazy<ColorConfig> = Lazy::new(|| {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let config_path = format!("{}/config/color_codes.json", manifest_dir);
    
    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("解析颜色配置文件失败: {}", e);
                ColorConfig {
                    color_map: HashMap::new(),
                    color_name: HashMap::new(),
                }
            })
        }
        Err(e) => {
            eprintln!("读取颜色配置文件失败: {}，使用默认配置", e);
            // 返回默认配置
            ColorConfig {
                color_map: create_default_color_map(),
                color_name: create_default_color_name(),
            }
        }
    }
});

// 创建默认的颜色映射（作为后备）
fn create_default_color_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("BL".to_string(), "Black".to_string());
    map.insert("BE".to_string(), "Blue".to_string());
    map.insert("RD".to_string(), "Red".to_string());
    map.insert("WH".to_string(), "White".to_string());
    map.insert("GR".to_string(), "Green".to_string());
    map.insert("YE".to_string(), "Yellow".to_string());
    map.insert("OR".to_string(), "Orange".to_string());
    map.insert("PI".to_string(), "Pink".to_string());
    map.insert("PR".to_string(), "Purple".to_string());
    map.insert("GY".to_string(), "Grey".to_string());
    map.insert("BR".to_string(), "Brown".to_string());
    map
}

// 创建默认的颜色名称（作为后备）
fn create_default_color_name() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("BL".to_string(), "Black".to_string());
    map.insert("BE".to_string(), "Blue".to_string());
    map.insert("RD".to_string(), "Red".to_string());
    map.insert("WH".to_string(), "White".to_string());
    map.insert("GR".to_string(), "Green".to_string());
    map.insert("YE".to_string(), "Yellow".to_string());
    map.insert("OR".to_string(), "Orange".to_string());
    map.insert("PI".to_string(), "Pink".to_string());
    map.insert("PR".to_string(), "Purple".to_string());
    map.insert("GY".to_string(), "Grey".to_string());
    map.insert("BR".to_string(), "Brown".to_string());
    map
}

// 获取颜色分类（用于分组）
pub fn get_color_map(color_code: &str) -> String {
    let code = extract_base_color_code(color_code);
    COLOR_CONFIG
        .color_map
        .get(&code)
        .cloned()
        .unwrap_or_else(|| "Multi".to_string())
}

// 获取颜色名称（用于显示）
pub fn get_color_name(color_code: &str) -> String {
    let code = extract_base_color_code(color_code);
    
    // 处理 SKU 前缀的特殊情况
    if code.starts_with("SKU") {
        return code.replace("SKU", "S");
    }
    
    COLOR_CONFIG
        .color_name
        .get(&code)
        .cloned()
        .unwrap_or_else(|| {
            if code.len() >= 2 && code.chars().all(|c| c.is_ascii_alphabetic()) {
                "Multi".to_string()
            } else {
                code
            }
        })
}

// 提取基础颜色代码（去除数字后缀）
// 例如：BL01 -> BL, RED123 -> RED
pub fn extract_base_color_code(color_code: &str) -> String {
    let code = color_code.to_uppercase();
    code.chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect()
}

// 从文件名提取颜色代码
pub fn extract_color_code_from_filename(filename: &str) -> Option<String> {
    // 去除扩展名
    let name_without_ext = filename.split('.').next()?;
    
    // 多种命名规则处理
    let parts: Vec<&str> = name_without_ext.split('_').collect();
    
    // 如果包含下划线，取第一部分
    let code = parts.first().unwrap_or(&name_without_ext);
    
    // 去除可能的前缀
    let code = code.trim_start_matches("color-")
                   .trim_start_matches("c-");
    
    if code.is_empty() {
        None
    } else {
        Some(code.to_uppercase())
    }
}

// 批量从图片提取颜色代码（去重）
pub fn extract_color_codes_from_images(images: &[crate::core::models::ProductImage]) -> Vec<String> {
    images.iter()
        .filter_map(|img| {
            img.original_name.as_ref()
                .and_then(|name| extract_color_code_from_filename(name))
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}