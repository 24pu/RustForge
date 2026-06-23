// src/presentation/handlers/product_export.rs

use axum::{
    extract::State,
    http::{header::{HeaderName, HeaderValue}, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use std::io::Cursor;
use chrono::Local;
use uuid::Uuid;
use std::collections::HashMap;

use crate::presentation::AppState;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::{check_permission, get_site_config_map, generate_size_table, size_table_to_simple_html, get_size_list_from_product, parse_size_table_data_with_unit,convert_size_to_standard};
use crate::core::{AmaTemplateRepository, ProductRepository, ProductAttributeValueRepository, AttributeTemplateRepository};
use crate::core::models::{AmaTemplate, Product, ProductVariant, ProductImage, ProductAttributeValue};
use crate::infrastructure::db::{
    PostgresProductAttributeValueRepo,
    PostgresAttributeTemplateRepo,
};
use rust_xlsxwriter::{Workbook, Format, Color};

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub product_ids: Vec<Uuid>,
}

// ========== 获取系统配置是否使用英寸 ==========
async fn get_use_inch(pool: &sqlx::PgPool) -> bool {
    let config_map = get_site_config_map(pool).await;
    config_map
        .get("product_size_inch")
        .map(|v| v == "true")
        .unwrap_or(false)
}

// ========== 获取单位字符串 ==========
async fn get_unit_string(pool: &sqlx::PgPool) -> String {
    let use_inch = get_use_inch(pool).await;
    if use_inch { "IN".to_string() } else { "CM".to_string() }
}

// ========== 解析包装信息 ==========
fn parse_package_info(product: &Product) -> (String, String, String, String) {
    let default_height = "0";
    let default_width = "0";
    let default_length = "0";
    let default_weight = "0";
    
    let (length, width, height) = if let Some(ref package) = product.package {
        let parts: Vec<&str> = package.split_whitespace().collect();
        if parts.len() >= 3 {
            (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            )
        } else {
            (default_length.to_string(), default_width.to_string(), default_height.to_string())
        }
    } else {
        (default_length.to_string(), default_width.to_string(), default_height.to_string())
    };
    
    let weight = product.weight.clone().unwrap_or(default_weight.to_string());
    
    (length, width, height, weight)
}

// ========== 获取包装字段值 ==========
fn get_package_field_value(
    field: &str,
    length: &str,
    width: &str,
    height: &str,
    weight: &str,
) -> String {
    match field {
        "package_length" => length.to_string(),
        "package_width" => width.to_string(),
        "package_height" => height.to_string(),
        "package_weight" => weight.to_string(),
        "package_length_unit_of_measure" => "CM".to_string(),
        "package_width_unit_of_measure" => "CM".to_string(),
        "package_height_unit_of_measure" => "CM".to_string(),
        "package_weight_unit_of_measure" => "GR".to_string(),
        _ => "".to_string(),
    }
}

// ========== 从尺码表提取 inseam_length 和 waist_size（支持英寸转换） ==========
async fn extract_size_data_from_table_with_unit(
    product: &Product,
    pool: &sqlx::PgPool,
) -> HashMap<String, (String, String)> {
    let mut result = HashMap::new();
    
    let size_list = get_size_list_from_product(product);
    if size_list.is_empty() {
        return result;
    }
    
    let use_inch = get_use_inch(pool).await;
    
    if let Some(ref dnote) = product.dnote {
        let size_table = parse_size_table_data_with_unit(dnote, &size_list, use_inch);
        
        let mut inseam_idx: Option<usize> = None;
        let mut waist_idx: Option<usize> = None;
        
        let inseam_keywords = ["inseam", "inseam_length", "内长", "裤内长", "inseam length"];
        let waist_keywords = ["waist", "waist_size", "腰围", "waist size"];
        
        for (i, header) in size_table.headers.iter().enumerate() {
            let header_lower = header.to_lowercase();
            for keyword in &inseam_keywords {
                if header_lower.contains(&keyword.to_lowercase()) {
                    inseam_idx = Some(i);
                    break;
                }
            }
            for keyword in &waist_keywords {
                if header_lower.contains(&keyword.to_lowercase()) {
                    waist_idx = Some(i);
                    break;
                }
            }
        }
        
        for row in &size_table.rows {
            let inseam_value = if let Some(idx) = inseam_idx {
                row.values.get(idx).and_then(|v| v.clone()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            let waist_value = if let Some(idx) = waist_idx {
                row.values.get(idx).and_then(|v| v.clone()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            result.insert(row.name.clone(), (inseam_value, waist_value));
        }
    }
    
    result
}

// ========== 获取变体的尺寸数据 ==========
fn get_variant_size_data(
    variant: &ProductVariant,
    size_data_map: &HashMap<String, (String, String)>,
) -> (String, String) {
    let size = variant.size.as_deref().unwrap_or("");
    
    // 精确匹配
    if let Some((inseam, waist)) = size_data_map.get(size) {
        return (inseam.clone(), waist.clone());
    }
    
    // 不区分大小写匹配
    for (key, (inseam, waist)) in size_data_map {
        if key.to_lowercase() == size.to_lowercase() {
            return (inseam.clone(), waist.clone());
        }
    }
    
    ("".to_string(), "".to_string())
}

// 获取产品字段值（非图片列）- 主体行
fn get_product_field_value(
    product: &Product, 
    field: &str, 
    site_url: &str,
    size_table_html: Option<&str>,
) -> String {
    match field {
        "item_sku" => product.sku.clone().unwrap_or_default(),
        "item_name" => product.fullname.clone().unwrap_or_default(),
        "external_product_id" => product.slug.clone(),
        "brand_name" => product.brand.clone().unwrap_or_default(),
        "product_description" => size_table_html.unwrap_or(&product.description.clone().unwrap_or_default()).to_string(),
        "price" => product.price.clone().unwrap_or_default(),
        "list_price" => product.price.clone().unwrap_or_default(),
        "currency" => "".to_string(),
        "quantity" => product.stock.clone().unwrap_or_else(|| "0".to_string()),
        "main_image_url" => String::new(),
        "swatch_image_url" => String::new(),
        "parent_child" => "parent".to_string(),
        "parent_sku" => "".to_string(),
        // 包装字段 - 主体不导出包装信息，返回空
        "package_length" | "package_width" | "package_height" | "package_weight" |
        "package_length_unit_of_measure" | "package_width_unit_of_measure" | 
        "package_height_unit_of_measure" | "package_weight_unit_of_measure" => "".to_string(),
        // 尺码字段 - 主体不导出
        "inseam_length" | "inseam_length_unit_of_measure" | 
        "waist_size" | "waist_size_unit_of_measure" => "".to_string(),
        // relationship_type - 主体为空
        "relationship_type" => "".to_string(),
        // variation_theme - 主体和变体都填 color-size
        "variation_theme" => "color-size".to_string(),
        // size_name - 主体为空
        "size_name" => "".to_string(),
        // size_map - 主体为空
        "size_map" => "".to_string(),
        _ => "".to_string(),
    }
}

// 获取变体字段值
fn get_variant_field_value(
    variant: &ProductVariant,
    field: &str,
    parent_sku: &str,
    product_fullname: &str,
    brand: &str,
    _site_url: &str,
    size_table_html: Option<&str>,
    length: &str,
    width: &str,
    height: &str,
    weight: &str,
    inseam: &str,
    waist: &str,
    unit: &str,
) -> String {
    // 先检查是否是包装字段
    let package_fields = [
        "package_length", "package_width", "package_height", "package_weight",
        "package_length_unit_of_measure", "package_width_unit_of_measure", 
        "package_height_unit_of_measure", "package_weight_unit_of_measure"
    ];
    if package_fields.contains(&field) {
        return get_package_field_value(field, length, width, height, weight);
    }
    
    // 获取尺码值并转换
    let raw_size = variant.size.as_deref().unwrap_or("");
    let standard_size = convert_size_to_standard(raw_size);
    
    match field {
        "item_sku" => variant.sku.clone(),
        "item_name" => {
            let color_part = variant.color_name.as_deref().unwrap_or("");
            let size_part = standard_size.to_uppercase();
            let parts = [product_fullname, color_part, &size_part];
            let name = parts
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            name
        }
        "brand_name" => brand.to_string(),
        "color_name" => variant.color_name.clone().unwrap_or_default(),
        "size" => raw_size.to_string(),
        "price" => variant.price.map(|p| p.to_string()).unwrap_or_default(),
        "quantity" => variant.stock.to_string(),
        "parent_child" => "child".to_string(),
        "parent_sku" => parent_sku.to_string(),
        "product_description" => size_table_html.unwrap_or("").to_string(),
        "main_image_url" => String::new(),
        "swatch_image_url" => String::new(),
        // 尺码字段 - 使用动态单位
        "inseam_length" => inseam.to_string(),
        "inseam_length_unit_of_measure" => unit.to_string(),
        "waist_size" => waist.to_string(),
        "waist_size_unit_of_measure" => unit.to_string(),
        // relationship_type - 变体为 Variation
        "relationship_type" => "Variation".to_string(),
        // variation_theme - 变体为 color-size
        "variation_theme" => "color-size".to_string(),
        // size_name - 使用转换后的标准尺码
        "size_name" => standard_size.clone(),
        // size_map - 使用转换后的标准尺码
        "size_map" => standard_size,
        _ => "".to_string(),
    }
}


// 将图片URL转换为绝对路径
fn make_absolute_url(url: &str, site_url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    if !site_url.is_empty() && !url.starts_with("http") && !url.starts_with("//") {
        format!("{}{}", site_url, url)
    } else {
        url.to_string()
    }
}

// 按颜色代码分组加载产品图片
async fn load_product_images_by_color(
    state: &AppState,
    product_id: Uuid,
) -> Result<HashMap<String, Vec<ProductImage>>, anyhow::Error> {
    let images = state.product_repo.get_product_images(product_id, None).await?;
    let mut map: HashMap<String, Vec<ProductImage>> = HashMap::new();
    for img in images {
        let color_code = img.color_code.clone().unwrap_or_else(|| "default".to_string());
        map.entry(color_code).or_default().push(img);
    }
    Ok(map)
}

// 为产品构建图片字段值（主图、色板图、其他图片）
struct ProductImageFields {
    main_image: String,
    swatch_image: String,
    other_images: Vec<String>, // 最多8个
}

fn build_product_image_fields(product: &Product, site_url: &str) -> ProductImageFields {
    let main_image = make_absolute_url(product.cover_image.as_deref().unwrap_or(""), site_url);
    let swatch_image = main_image.clone();
    ProductImageFields {
        main_image,
        swatch_image,
        other_images: vec!["".to_string(); 8],
    }
}

// 辅助函数：从完整颜色代码中提取基础代码（去掉尾部数字）
fn extract_base_color_code(full_code: &str) -> String {
    let mut end = full_code.len();
    for (i, ch) in full_code.chars().rev().enumerate() {
        if !ch.is_ascii_digit() {
            end = full_code.len() - i;
            break;
        }
    }
    full_code[..end].to_string()
}

async fn build_variant_image_fields(
    state: &AppState,
    product_id: Uuid,
    variant: &ProductVariant,
    site_url: &str,
) -> ProductImageFields {
    let images_by_color = match load_product_images_by_color(state, product_id).await {
        Ok(map) => map,
        Err(e) => {
            eprintln!("加载产品图片失败 product {}: {}", product_id, e);
            HashMap::new()
        }
    };
    let color_code = variant.color_code.as_deref().unwrap_or("default");
    let mut images = images_by_color.get(color_code).cloned().unwrap_or_default();

    if images.is_empty() {
        let base_code = extract_base_color_code(color_code);
        if base_code != color_code {
            images = images_by_color.get(&base_code).cloned().unwrap_or_default();
        }
    }

    let urls: Vec<String> = images.iter().map(|img| make_absolute_url(&img.url, site_url)).collect();
    let main_image = urls.first().cloned().unwrap_or_default();
    let swatch_image = main_image.clone();
    let other_images = if urls.len() > 1 { urls[1..].to_vec() } else { vec![] };
    let mut other_fixed = vec!["".to_string(); 8];
    for (i, url) in other_images.iter().enumerate().take(8) {
        other_fixed[i] = url.clone();
    }
    ProductImageFields {
        main_image,
        swatch_image,
        other_images: other_fixed,
    }
}

// ========== 加载产品属性 ==========
async fn load_product_attributes_by_product(
    state: &AppState,
    product_id: Uuid,
) -> Result<HashMap<String, String>, anyhow::Error> {
    let value_repo = PostgresProductAttributeValueRepo::new(state.db_pool.clone());
    let template_repo = PostgresAttributeTemplateRepo::new(state.db_pool.clone());
    
    let values = value_repo.get_product_attribute_values(product_id).await?;
    let mut result = HashMap::new();
    
    for val in values {
        if let Some(template) = template_repo.get_template_by_id(val.attribute_template_id).await? {
            let key = template.title.clone().unwrap_or(template.name);
            result.insert(key, val.value.clone().unwrap_or_default());
        }
    }
    
    Ok(result)
}

// ========== 生成尺码表 HTML ==========
fn generate_size_table_html(product: &Product) -> String {
    if let Some(ref dnote) = product.dnote {
        let size_table = generate_size_table(dnote, product);
        let html = size_table_to_simple_html(&size_table);
        if !html.is_empty() {
            return html;
        }
    }
    product.description.clone().unwrap_or_default()
}

// ========== 构建带属性的产品行 ==========
fn build_product_row_with_attributes(
    columns: &[String],
    product: &Product,
    images: &ProductImageFields,
    site_url: &str,
    attribute_map: &HashMap<String, String>,
    size_table_html: Option<&str>,
) -> Vec<String> {
    columns
        .iter()
        .map(|col| {
            // 首先检查是否是属性列（匹配模板标题）
            if let Some(attr_value) = attribute_map.get(col.as_str()) {
                return attr_value.clone();
            }
            
            match col.as_str() {
                "main_image_url" => images.main_image.clone(),
                "swatch_image_url" => images.swatch_image.clone(),
                c if c.starts_with("other_image_url") => {
                    let idx = c.trim_start_matches("other_image_url").parse::<usize>().unwrap_or(1);
                    if idx >= 1 && idx <= 8 {
                        images.other_images.get(idx - 1).cloned().unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                _ => get_product_field_value(product, col, site_url, size_table_html),
            }
        })
        .collect()
}

// ========== 构建带属性的变体行 ==========
fn build_variant_row_with_attributes(
    columns: &[String],
    variant: &ProductVariant,
    parent_sku: &str,
    product_fullname: &str,
    brand: &str,
    images: &ProductImageFields,
    site_url: &str,
    attribute_map: &HashMap<String, String>,
    size_table_html: Option<&str>,
    length: &str,
    width: &str,
    height: &str,
    weight: &str,
    inseam: &str,
    waist: &str,
    unit: &str,
) -> Vec<String> {
    columns
        .iter()
        .map(|col| {
            // 首先检查是否是属性列（匹配模板标题）
            if let Some(attr_value) = attribute_map.get(col.as_str()) {
                return attr_value.clone();
            }
            
            match col.as_str() {
                "main_image_url" => images.main_image.clone(),
                "swatch_image_url" => images.swatch_image.clone(),
                c if c.starts_with("other_image_url") => {
                    let idx = c.trim_start_matches("other_image_url").parse::<usize>().unwrap_or(1);
                    if idx >= 1 && idx <= 8 {
                        images.other_images.get(idx - 1).cloned().unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                _ => get_variant_field_value(
                    variant, col, parent_sku, product_fullname, brand, site_url, 
                    size_table_html, length, width, height, weight, inseam, waist, unit
                ),
            }
        })
        .collect()
}

async fn get_active_template_for_user(state: &AppState, user_id: Uuid) -> Result<Option<AmaTemplate>, anyhow::Error> {
    let templates = state.amatemplate_repo.list(user_id).await?;
    let active = templates
        .into_iter()
        .filter(|t| t.is_used)
        .max_by_key(|t| t.created_at);
    Ok(active)
}

async fn create_default_template(state: &AppState, user_id: Uuid) -> Result<(), anyhow::Error> {
    let default_content = include_str!("../../../config/default_amatemplate.json");
    let json: serde_json::Value = serde_json::from_str(default_content)?;
    let template_text = json["DEFAULT_T"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("默认模板内容缺失"))?;
    let name = format!("默认模板_{}", Local::now().format("%Y%m%d_%H%M%S"));
    state
        .amatemplate_repo
        .create(&name, template_text, true, user_id)
        .await?;
    Ok(())
}

fn generate_excel(rows: &[Vec<String>]) -> Result<Vec<u8>, anyhow::Error> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new()
        .set_background_color(Color::Yellow)
        .set_bold();

    let col_count = rows.first().map_or(0, |r| r.len());
    let mut max_widths = vec![0; col_count];

    for row in rows.iter() {
        for (col_idx, cell) in row.iter().enumerate() {
            let len = cell.chars().count();
            if len > max_widths[col_idx] {
                max_widths[col_idx] = len;
            }
        }
    }

    let extra_padding = 2;
    for (col_idx, &max_len) in max_widths.iter().enumerate() {
        let width = (max_len + extra_padding) as f64;
        if col_idx <= 3 {
            worksheet.set_column_width(col_idx as u16, width)?;
        }
    }

    let row_height = 28.0;
    let pt_row_height = 24.0;
    
    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            if row_idx == 0 {
                worksheet.set_row_height(row_idx as u32, row_height)?;
                worksheet.write_string_with_format(row_idx as u32, col_idx as u16, cell, &header_format)?;
            } else {
                worksheet.set_row_height(row_idx as u32, pt_row_height)?;
                worksheet.write_string(row_idx as u32, col_idx as u16, cell)?;
            }
        }
    }

    let mut buffer = Cursor::new(Vec::new());
    workbook.save_to_writer(&mut buffer)?;
    Ok(buffer.into_inner())
}

pub async fn export_selected_products_handler(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExportRequest>,
) -> impl IntoResponse {
    let user_id = match user_opt {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    if let Err((status, msg)) = check_permission(Some(user_id), &state.user_repo, "product:list").await {
        return (status, msg).into_response();
    }

    // 获取启用的模板
    let template = match get_active_template_for_user(&state, user_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            if let Err(e) = create_default_template(&state, user_id).await {
                eprintln!("创建默认模板失败: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "模板初始化失败").into_response();
            }
            match get_active_template_for_user(&state, user_id).await {
                Ok(Some(t)) => t,
                _ => return (StatusCode::INTERNAL_SERVER_ERROR, "模板不可用").into_response(),
            }
        }
        Err(e) => {
            eprintln!("获取模板失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "获取模板失败").into_response();
        }
    };

    let columns: Vec<String> = template
        .value
        .split_whitespace()
        .map(|s| s.trim().to_string())
        .collect();
    if columns.is_empty() {
        return (StatusCode::BAD_REQUEST, "模板无有效列定义").into_response();
    }

    // 获取站点URL
    let config_map = get_site_config_map(&state.db_pool).await;
    let site_url = config_map.get("site_url").cloned().unwrap_or_default();

    // 获取单位（CM 或 IN）
    let unit = get_unit_string(&state.db_pool).await;

    // 获取产品列表
    let mut products = Vec::new();
    for product_id in payload.product_ids {
        match state.product_repo.get_product_by_id(product_id).await {
            Ok(Some(p)) => products.push(p),
            _ => continue,
        }
    }

    let mut rows = Vec::new();
    rows.push(columns.clone()); // 表头

    for product in &products {
        // ========== 解析包装信息 ==========
        let (length, width, height, weight) = parse_package_info(product);
        
        // ========== 提取尺码数据（inseam_length 和 waist_size，带单位转换） ==========
        let size_data_map = extract_size_data_from_table_with_unit(product, &state.db_pool).await;
        
        // ========== 生成尺码表 HTML ==========
        let size_table_html = generate_size_table_html(product);
        let size_table_html_ref = if size_table_html.is_empty() {
            None
        } else {
            Some(size_table_html.as_str())
        };
        
        // ========== 加载产品属性 ==========
        let attribute_map = match load_product_attributes_by_product(&state, product.id).await {
            Ok(map) => map,
            Err(e) => {
                eprintln!("加载产品属性失败 product {}: {}", product.id, e);
                HashMap::new()
            }
        };
        
        // ========== 产品主行 ==========
        let product_images = build_product_image_fields(product, &site_url);
        let product_row = build_product_row_with_attributes(
            &columns, 
            product, 
            &product_images, 
            &site_url,
            &attribute_map,
            size_table_html_ref,
        );
        rows.push(product_row);

        // ========== 变体行 ==========
        let variants = match state.product_repo.list_variants(product.id).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("获取变体失败 product {}: {}", product.id, e);
                continue;
            }
        };
        let parent_sku = product.sku.clone().unwrap_or_default();
        for variant in &variants {
            // 获取该变体的 inseam_length 和 waist_size
            let (inseam, waist) = get_variant_size_data(variant, &size_data_map);
            
            let variant_images = build_variant_image_fields(&state, product.id, variant, &site_url).await;
            let variant_row = build_variant_row_with_attributes(
                &columns,
                variant,
                &parent_sku,
                &product.fullname.as_deref().unwrap_or_default(),
                &product.brand.as_deref().unwrap_or_default(),
                &variant_images,
                &site_url,
                &attribute_map,
                size_table_html_ref,
                &length,
                &width,
                &height,
                &weight,
                &inseam,
                &waist,
                &unit,
            );
            rows.push(variant_row);
        }
    }

    let excel_data = match generate_excel(&rows) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("生成Excel失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "生成Excel失败").into_response();
        }
    };

    let filename = format!("products_{}.xlsx", Local::now().format("%Y%m%d_%H%M%S"));
    let mut response = Response::new(excel_data.into());
    response.headers_mut().insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
    );
    response.headers_mut().insert(
        HeaderName::from_static("content-disposition"),
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
    );
    response
}