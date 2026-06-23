// src/presentation/handlers/utils.rs

use axum::http::StatusCode;
use uuid::Uuid;
use std::sync::Arc;
use std::path::Path;
use image;
use anyhow::Result;

use crate::core::UserRepository;
use crate::presentation::AppState;
use crate::core::models::Category;

use std::collections::HashMap;
use sqlx::PgPool;
use serde_json::Value;

use serde::{Deserialize, Serialize};

pub async fn check_permission(
    user_id: Option<Uuid>,
    repo: &Arc<dyn UserRepository>,
    perm: &str,
) -> Result<(), (StatusCode, &'static str)> {
    let uid = user_id.ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))?;
    let has = repo
        .user_has_permission(uid, perm)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"))?;
    if !has {
        Err((StatusCode::FORBIDDEN, "Forbidden"))
    } else {
        Ok(())
    }
}

pub async fn get_config_value(pool: &sqlx::PgPool, key: &str) -> Result<String, String> {
    match sqlx::query!("SELECT value FROM site_config WHERE key = $1", key)
        .fetch_one(pool)
        .await
    {
        Ok(row) => Ok(row.value),
        Err(sqlx::Error::RowNotFound) => Err(format!("Config key '{}' not found", key)),
        Err(e) => {
            eprintln!("Failed to get config value: {}", e);
            Err("Database error".to_string())
        }
    }
}

pub async fn get_nav_categories(state: &Arc<AppState>) -> Vec<Category> {
    let categories = state.content_repo.list_categories_tree(None).await.unwrap_or_default();
    categories.into_iter()
        .filter(|c| c.parent_id.is_none() && c.show_in_nav)
        .map(|mut c| {
            if let Some(children) = &mut c.children {
                children.retain(|child| child.show_in_nav);
            }
            c
        })
        .collect()
}

pub fn generate_thumbnail(src: &Path, dst: &Path, max_size: u32) -> Result<()> {
    let img = image::open(src)?;
    let (width, height) = (img.width(), img.height());
    let (nw, nh) = if width > height {
        (max_size, (max_size as f32 * height as f32 / width as f32) as u32)
    } else {
        ((max_size as f32 * width as f32 / height as f32) as u32, max_size)
    };
    let thumbnail = img.resize(nw, nh, image::imageops::FilterType::Lanczos3);
    thumbnail.save(dst)?;
    Ok(())
}

pub async fn get_site_config_map(pool: &PgPool) -> HashMap<String, String> {
    let mut config = HashMap::new();
    let rows = match sqlx::query!("SELECT key, value FROM site_config")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch site config: {}", e);
            return config;
        }
    };
    for row in rows {
        config.insert(row.key, row.value);
    }
    config.entry("seo_title".to_string()).or_insert("企业网站".to_string());
    config.entry("seo_description".to_string()).or_insert("".to_string());
    config.entry("seo_keywords".to_string()).or_insert("".to_string());
    config.entry("logo_url".to_string()).or_insert("".to_string());
    config.entry("favicon_url".to_string()).or_insert("/favicon.ico".to_string());
    config.entry("site_name".to_string()).or_insert("Enterprise".to_string());
    config
}

// ========== 尺码表格数据结构 ==========
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeTableRow {
    pub name: String,
    pub values: Vec<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeTable {
    pub headers: Vec<String>,
    pub rows: Vec<SizeTableRow>,
    pub unit: String,
}

/// 解析尺码数据文本，生成结构化的尺码表格
pub fn parse_size_table_data(raw_data: &str, size_list: &[String]) -> SizeTable {
    let lines: Vec<&str> = raw_data
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    
    if lines.is_empty() {
        return SizeTable {
            headers: size_list.to_vec(),
            rows: vec![],
            unit: "cm".to_string(),
        };
    }
    
    let part_names: Vec<String> = lines[0]
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    
    let mut data_rows: Vec<Vec<String>> = Vec::new();
    for line in &lines[1..] {
        let values: Vec<String> = line
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if !values.is_empty() {
            data_rows.push(values);
        }
    }
    
    let max_cols = if !data_rows.is_empty() {
        data_rows.iter().map(|row| row.len()).max().unwrap_or(0)
    } else {
        0
    };
    
    let final_size_list = if !size_list.is_empty() {
        size_list.to_vec()
    } else if max_cols > 0 {
        (1..=max_cols).map(|i| format!("尺码{}", i)).collect()
    } else {
        vec![]
    };
    
    let col_count = std::cmp::min(final_size_list.len(), max_cols);
    let trimmed_size_list: Vec<String> = final_size_list.iter().take(col_count).cloned().collect();
    
    let mut rows = Vec::new();
    
    for size_idx in 0..trimmed_size_list.len() {
        let size_name = trimmed_size_list[size_idx].clone();
        let mut row_values = Vec::new();
        
        for part_idx in 0..part_names.len() {
            if part_idx < data_rows.len() && size_idx < data_rows[part_idx].len() {
                row_values.push(Some(data_rows[part_idx][size_idx].clone()));
            } else {
                row_values.push(None);
            }
        }
        
        let has_data = row_values.iter().any(|v| v.is_some());
        if has_data {
            rows.push(SizeTableRow {
                name: size_name,
                values: row_values,
            });
        }
    }
    
    SizeTable {
        headers: part_names,
        rows,
        unit: "cm".to_string(),
    }
}

/// 解析尺码数据文本，生成结构化的尺码表格（支持厘米转英寸）
pub fn parse_size_table_data_with_unit(raw_data: &str, size_list: &[String], use_inch: bool) -> SizeTable {
    let lines: Vec<&str> = raw_data
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    
    if lines.is_empty() {
        return SizeTable {
            headers: size_list.to_vec(),
            rows: vec![],
            unit: if use_inch { "inch".to_string() } else { "cm".to_string() },
        };
    }
    
    let part_names: Vec<String> = lines[0]
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    
    let mut data_rows: Vec<Vec<String>> = Vec::new();
    for line in &lines[1..] {
        let values: Vec<String> = line
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if !values.is_empty() {
            data_rows.push(values);
        }
    }
    
    let max_cols = if !data_rows.is_empty() {
        data_rows.iter().map(|row| row.len()).max().unwrap_or(0)
    } else {
        0
    };
    
    let final_size_list = if !size_list.is_empty() {
        size_list.to_vec()
    } else if max_cols > 0 {
        (1..=max_cols).map(|i| format!("尺码{}", i)).collect()
    } else {
        vec![]
    };
    
    let col_count = std::cmp::min(final_size_list.len(), max_cols);
    let trimmed_size_list: Vec<String> = final_size_list.iter().take(col_count).cloned().collect();
    
    let mut rows = Vec::new();
    
    for size_idx in 0..trimmed_size_list.len() {
        let size_name = trimmed_size_list[size_idx].clone();
        let mut row_values = Vec::new();
        
        for part_idx in 0..part_names.len() {
            if part_idx < data_rows.len() && size_idx < data_rows[part_idx].len() {
                let value = data_rows[part_idx][size_idx].clone();
                let converted_value = if use_inch {
                    value.parse::<f64>().ok().map(|v| {
                        let inch = v / 2.54;
                        format!("{:.1}", inch)
                    }).unwrap_or(value)
                } else {
                    value
                };
                row_values.push(Some(converted_value));
            } else {
                row_values.push(None);
            }
        }
        
        let has_data = row_values.iter().any(|v| v.is_some());
        if has_data {
            rows.push(SizeTableRow {
                name: size_name,
                values: row_values,
            });
        }
    }
    
    SizeTable {
        headers: part_names,
        rows,
        unit: if use_inch { "inch".to_string() } else { "cm".to_string() },
    }
}

/// 从原始数据生成尺码表（一步完成，带单位转换）
pub async fn generate_size_table_with_unit(
    raw_data: &str, 
    product: &crate::core::models::Product,
    pool: &sqlx::PgPool
) -> SizeTable {
    let size_list = get_size_list_from_product(product);
    
    let config_map = get_site_config_map(pool).await;
    let use_inch = config_map
        .get("product_size_inch")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    parse_size_table_data_with_unit(raw_data, &size_list, use_inch)
}

/// 将尺码表格转换为 HTML 表格
pub fn size_table_to_html(table: &SizeTable) -> String {
    if table.headers.is_empty() || table.rows.is_empty() {
        return "<p class='text-gray-500'>暂无尺码数据</p>".to_string();
    }
    
    let mut html = String::from(
        "<div class='overflow-x-auto'><table class='min-w-full border-collapse border border-gray-300'>"
    );
    
    html.push_str("<thead><tr class='bg-gray-100'>");
    html.push_str("<th class='border border-gray-300 px-4 py-2 text-left font-semibold'>尺码</th>");
    for header in &table.headers {
        html.push_str(&format!(
            "<th class='border border-gray-300 px-4 py-2 text-left font-semibold'>{}</th>",
            escape_html(header)
        ));
    }
    html.push_str("</tr></thead>");
    
    html.push_str("<tbody>");
    for row in &table.rows {
        html.push_str("<tr>");
        html.push_str(&format!(
            "<td class='border border-gray-300 px-4 py-2 font-medium'>{}</td>",
            escape_html(&row.name)
        ));
        for value in &row.values {
            let display_value = value.as_ref().map(|v| escape_html(v)).unwrap_or_else(|| "-".to_string());
            html.push_str(&format!(
                "<td class='border border-gray-300 px-4 py-2 text-center'>{}</td>",
                display_value
            ));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></tr></div>");
    
    html
}

/// 将尺码表格转换为 JSON（供前端使用）
pub fn size_table_to_json(table: &SizeTable) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = table
        .rows
        .iter()
        .map(|row| {
            let values: Vec<String> = row
                .values
                .iter()
                .map(|v| v.as_ref().map(|s| s.clone()).unwrap_or_else(|| "-".to_string()))
                .collect();
            
            serde_json::json!({
                "size": row.name,
                "values": values
            })
        })
        .collect();
    
    serde_json::json!({
        "headers": table.headers,
        "rows": rows,
        "unit": table.unit
    })
}

/// HTML 转义
fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

/// 从产品获取尺码列表（支持空格和逗号分隔）
pub fn get_size_list_from_product(product: &crate::core::models::Product) -> Vec<String> {
    if let Some(ref csize) = product.csize {
        let sizes: Vec<String> = if csize.contains(',') {
            csize.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            csize.split_whitespace()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        if !sizes.is_empty() {
            return sizes;
        }
    }
    
    if let Some(ref ussize) = product.ussize {
        let sizes: Vec<String> = if ussize.contains(',') {
            ussize.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            ussize.split_whitespace()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        if !sizes.is_empty() {
            return sizes;
        }
    }
    
    if let Some(ref asize) = product.asize {
        let sizes: Vec<String> = if asize.contains(',') {
            asize.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            asize.split_whitespace()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        if !sizes.is_empty() {
            return sizes;
        }
    }
    
    vec![]
}

/// 从原始数据生成尺码表（一步完成）
pub fn generate_size_table(raw_data: &str, product: &crate::core::models::Product) -> SizeTable {
    let size_list = get_size_list_from_product(product);
    parse_size_table_data(raw_data, &size_list)
}

// ============================================================
// 尺码表 HTML 导出（纯文本/P 标签格式）
// 用于 Excel 导出中的 product_description 字段
// ============================================================

/// 将尺码表转换为简单的 HTML 格式（只用 <p> 标签）
/// 用于 Excel 导出中的 product_description 字段
pub fn size_table_to_simple_html(table: &SizeTable) -> String {
    if table.headers.is_empty() || table.rows.is_empty() {
        return String::new();
    }
    
    let mut html = String::new();
    
    // 添加标题
    html.push_str("<p><strong>Size Chart</strong></p>");
    
    // 表头行
    let mut header_parts = vec!["Size".to_string()];
    for header in &table.headers {
        header_parts.push(header.clone());
    }
    html.push_str(&format!("<p>{}</p>", header_parts.join(" | ")));
    
    // 分隔线
    let separator: Vec<String> = (0..header_parts.len()).map(|_| "---".to_string()).collect();
    html.push_str(&format!("<p>{}</p>", separator.join(" | ")));
    
    // 数据行
    for row in &table.rows {
        let mut row_parts = vec![row.name.clone()];
        for value in &row.values {
            let display = value.as_ref().map(|v| v.as_str()).unwrap_or("-");
            row_parts.push(display.to_string());
        }
        html.push_str(&format!("<p>{}</p>", row_parts.join(" | ")));
    }
    
    // 添加单位信息
    if !table.unit.is_empty() {
        html.push_str(&format!("<p>Unit: {}</p>", table.unit));
    }
    
    html
}

/// 将尺码表转换为纯文本表格（用于导出）
pub fn size_table_to_plain_text(table: &SizeTable) -> String {
    if table.headers.is_empty() || table.rows.is_empty() {
        return String::new();
    }
    
    let mut lines = Vec::new();
    
    // 表头
    let mut header_parts = vec!["尺码".to_string()];
    for header in &table.headers {
        header_parts.push(header.clone());
    }
    lines.push(header_parts.join("\t"));
    
    // 数据行
    for row in &table.rows {
        let mut row_parts = vec![row.name.clone()];
        for value in &row.values {
            let display = value.as_ref().map(|v| v.as_str()).unwrap_or("-");
            row_parts.push(display.to_string());
        }
        lines.push(row_parts.join("\t"));
    }
    
    // 添加单位
    if !table.unit.is_empty() {
        lines.push(format!("单位: {}", table.unit));
    }
    
    lines.join("\n")
}

// ========== 尺码转换函数 ==========
pub fn convert_size_to_standard(size: &str) -> String {
    if size.is_empty() {
        return String::new();
    }
    
    let size_lower = size.to_lowercase().trim().to_string();
    
    match size_lower.as_str() {
        "xs" | "x-small" => "X-Small".to_string(),
        "s" | "small" => "Small".to_string(),
        "m" | "medium" => "Medium".to_string(),
        "l" | "large" => "Large".to_string(),
        "xl" | "x-large" => "X-Large".to_string(),
        "2xl" | "xxl" | "xx-large" => "XX-Large".to_string(),
        "3xl" | "xxxl" | "xxx-large" => "XXX-Large".to_string(),
        // 如果已经是大写格式，直接返回
        "x-small" => "X-Small".to_string(),
        "xx-small" => "XX-Small".to_string(),
        "xxx-small" => "XXX-Small".to_string(),
        _ => {
            // 如果已经是标准格式（包含 -），直接返回
            if size.contains('-') {
                // 首字母大写
                let parts: Vec<&str> = size.split('-').collect();
                if parts.len() == 2 {
                    let first = parts[0].trim();
                    let second = parts[1].trim();
                    if !first.is_empty() && !second.is_empty() {
                        let first_capitalized = first.chars().next().unwrap_or('X').to_uppercase().collect::<String>() + &first[1..].to_lowercase();
                        let second_capitalized = second.chars().next().unwrap_or('S').to_uppercase().collect::<String>() + &second[1..].to_lowercase();
                        return format!("{}-{}", first_capitalized, second_capitalized);
                    }
                }
                size.to_string()
            } else {
                // 其他情况：首字母大写
                let mut chars = size.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => size.to_string(),
                }
            }
        }
    }
}