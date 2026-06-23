// handlers/product_admin.rs

use axum::{
    extract::{State, Path, Multipart, Query},  // 添加 Query
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;  // 添加 HashMap
use uuid::Uuid;
use chrono::Utc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::core::ProductCategoryRepository;
use crate::core::ProductRepository;
use crate::core::{CreateProductInput, UpdateProductInput, Pagination, ProductFilters, CreateVariantInput, UpdateVariantInput};
use serde::{Deserialize, Serialize};
use crate::infrastructure::color_code;

#[derive(Debug, Deserialize)]
pub struct VariantGenerateRule {
    pub sku_pattern: String,        // SKU生成规则，如 "{product_sku}{color_code}_{size}"
    pub color_code_field: String,   // 颜色代码来源: "filename" 或 "color_list"
    pub size_field: String,         // 尺码来源: "csize", "ussize", "asize"
    pub default_price: Option<f64>,
    pub default_stock: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct VariantInfo {
    pub id: Uuid,
    pub sku: String,
    pub color: Option<String>,
    pub color_name: Option<String>,
    pub size: Option<String>,
    pub price: Option<f64>,
    pub stock: i32,
    pub is_default: bool,
}

// ========== 产品分类 CRUD ==========

pub async fn list_product_categories_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.product_category_repo.list_categories_tree(None).await {
        Ok(tree) => (StatusCode::OK, Json(tree)).into_response(),
        Err(e) => {
            eprintln!("获取分类树失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

pub async fn create_product_category_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProductCategoryRequest>,
) -> impl IntoResponse {
    match state.product_category_repo.create_category(
        &payload.name,
        &payload.slug,
        payload.description.as_deref(),
        payload.parent_id,
    ).await {
        Ok(category) => (
            StatusCode::CREATED,
            Json(json!({
                "id": category.id,
                "name": category.name,
                "slug": category.slug,
                "description": category.description,
                "parent_id": category.parent_id,
                "sort": category.sort,
                "show_in_nav": category.show_in_nav,
                "created_at": category.created_at,
                "updated_at": category.updated_at,
            })),
        ).into_response(),
        Err(e) => {
            eprintln!("创建产品分类失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("创建失败: {}", e)})),
            ).into_response()
        }
    }
}

pub async fn get_product_category_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.product_category_repo.get_category_by_id(id).await {
        Ok(Some(category)) => (StatusCode::OK, Json(category)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "分类不存在"})),
        ).into_response(),
        Err(e) => {
            eprintln!("获取分类失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

pub async fn update_product_category_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateProductCategoryRequest>,
) -> impl IntoResponse {
    match state.product_category_repo.update_category(
        id,
        &payload.name,
        &payload.slug,
        payload.description.as_deref(),
        payload.parent_id,
    ).await {
        Ok(category) => (StatusCode::OK, Json(category)).into_response(),
        Err(e) => {
            eprintln!("更新分类失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

pub async fn delete_product_category_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.product_category_repo.delete_category(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "分类不存在"})),
        ).into_response(),
        Err(e) => {
            eprintln!("删除分类失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

// ========== 产品 CRUD ==========

// ========== 创建产品 ==========
pub async fn create_product_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProductRequest>,
) -> impl IntoResponse {
    println!("收到创建产品请求: {:?}", payload);
    println!("选中的分类IDs: {:?}", payload.category_ids);
    
    // 验证 slug 唯一性
    match state.product_repo.get_product_by_slug(&payload.slug).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({"error": "产品 slug 已存在"})),
            ).into_response();
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("检查 slug 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("检查失败: {}", e)})),
            ).into_response();
        }
    }

    // 转换请求为 CreateProductInput
    let input = CreateProductInput {
        sku: payload.sku.clone(),
        slug: payload.slug.clone(),
        lang: Some(payload.lang.clone().unwrap_or_else(|| "zh".to_string())),
        name: payload.name.clone(),
        dname: payload.dname.clone(),
        fullname: payload.fullname.clone(),
        brand: payload.brand.clone(),
        cover_image: payload.cover_image.clone(),
        summary: payload.summary.clone(),
        description: payload.description.clone(),
        keywords: payload.keywords.clone(),
        points: payload.points.clone(),
        dnote: payload.dnote.clone(),
        csize: payload.csize.clone(),
        ussize: payload.ussize.clone(),
        asize: payload.asize.clone(),
        fabric_type: payload.fabric_type.clone(),
        price: payload.price.clone(),
        stock: payload.stock.clone(),
        package: payload.package.clone(),
        weight: payload.weight.clone(),
        published: Some(payload.published.unwrap_or(false)),
        translation_group: payload.translation_group,
        user_id: payload.user_id,
        size_list: payload.size_list.clone(),
        color_list: payload.color_list.clone(),
        color_names: payload.color_names.clone(),
    };

    match state.product_repo.create_product(input).await {
        Ok(product) => {
            println!("产品创建成功: {:?}", product.id);
            
            // 保存产品分类关联（支持多个分类）- 使用引用避免移动所有权
            if let Some(ref category_ids) = payload.category_ids {
                if !category_ids.is_empty() {
                    println!("准备保存分类关联: product_id={}, category_ids={:?}", product.id, category_ids);
                    
                    for &category_id in category_ids {
                        let _ = sqlx::query!(
                            "INSERT INTO product_category_relations (product_id, category_id) VALUES ($1, $2)",
                            product.id, category_id
                        )
                        .execute(&state.db_pool)
                        .await;
                    }
                    println!("分类关联保存成功，共 {} 个分类", category_ids.len());
                }
            } else {
                println!("没有选择分类");
            }
            
            (StatusCode::CREATED, Json(json!({
                "id": product.id,
                "name": product.name,
                "slug": product.slug,
                "published": product.published,
                "message": "产品创建成功"
            }))).into_response()
        }
        Err(e) => {
            eprintln!("创建产品失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("创建失败: {}", e)})),
            ).into_response()
        }
    }
}

// ========== 更新产品 ==========
pub async fn update_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> impl IntoResponse {
    println!("更新产品: id={}, payload={:?}", id, payload);
    println!("选中的分类IDs: {:?}", payload.category_ids);
    
    let input = UpdateProductInput {
        name: payload.name.clone(),
        sku: payload.sku.clone(),
        dname: payload.dname.clone(),
        fullname: payload.fullname.clone(),
        brand: payload.brand.clone(),
        cover_image: payload.cover_image.clone(),
        summary: payload.summary.clone(),
        description: payload.description.clone(),
        keywords: payload.keywords.clone(),
        points: payload.points.clone(),
        dnote: payload.dnote.clone(),
        csize: payload.csize.clone(),
        ussize: payload.ussize.clone(),
        asize: payload.asize.clone(),
        fabric_type: payload.fabric_type.clone(),
        price: payload.price.clone(),
        stock: payload.stock.clone(),
        package: payload.package.clone(),
        weight: payload.weight.clone(),
        published: payload.published,
        translation_group: None,
        user_id: None,
        size_list: payload.size_list.clone(),
        color_list: payload.color_list.clone(),
        color_names: payload.color_names.clone(),
    };
    
    match state.product_repo.update_product(id, input).await {
        Ok(product) => {
            // 更新分类关联（支持多个分类）- 使用引用避免移动所有权
            if let Some(ref category_ids) = payload.category_ids {
                println!("准备更新分类: product_id={}, category_ids={:?}", id, category_ids);
                
                // 先删除旧的关联
                let _ = sqlx::query!(
                    "DELETE FROM product_category_relations WHERE product_id = $1",
                    id
                )
                .execute(&state.db_pool)
                .await;
                
                // 插入新的关联
                for &category_id in category_ids {
                    let _ = sqlx::query!(
                        "INSERT INTO product_category_relations (product_id, category_id) VALUES ($1, $2)",
                        id, category_id
                    )
                    .execute(&state.db_pool)
                    .await;
                }
                
                println!("产品分类已更新完成，共 {} 个分类", category_ids.len());
            }
            
            (StatusCode::OK, Json(product)).into_response()
        }
        Err(e) => {
            eprintln!("更新产品失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}



pub async fn list_products_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // 解析分页参数
    let page = params.get("page")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);
    let per_page = params.get("per_page")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(20);
    
    // 解析筛选参数
    let keyword = params.get("keyword").cloned();
    let category_id = params.get("category_id")
        .and_then(|c| c.parse::<i32>().ok());
    let published = params.get("published")
        .and_then(|p| p.parse::<bool>().ok());
    
    let pagination = Pagination::new(page, per_page);
    let filters = ProductFilters {
        keyword,
        category_id,
        published,
    };
    
    match state.product_repo.list_products(pagination, filters).await {
        Ok((products, total)) => {
            let total_pages = (total + per_page as i64 - 1) / per_page as i64;
            let response = json!({
                "products": products,
                "total": total,
                "page": page,
                "per_page": per_page,
                "total_pages": total_pages
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            eprintln!("获取产品列表失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

pub async fn get_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.product_repo.get_product_by_id(id).await {
        Ok(Some(product)) => (StatusCode::OK, Json(product)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "产品不存在"})),
        ).into_response(),
        Err(e) => {
            eprintln!("获取产品失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}



pub async fn delete_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.product_repo.delete_product(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "产品不存在"})),
        ).into_response(),
        Err(e) => {
            eprintln!("删除产品失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}









// 上传产品图片
pub async fn upload_product_images_handler(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 创建产品图片目录
    let upload_dir = PathBuf::from(format!("uploads/products/{}", product_id));
    if !upload_dir.exists() {
        if let Err(e) = fs::create_dir_all(&upload_dir).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("创建目录失败: {}", e)})),
            ).into_response();
        }
    }

    let mut uploaded_images = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        // 检查文件类型
        let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        if !mime_type.starts_with("image/") {
            continue;
        }

        // 从文件名提取颜色代码
        let color_code = extract_color_code_from_filename(&file_name);

        // 生成唯一文件名
        let ext = std::path::Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let unique_name = format!("{}_{}.{}", timestamp, Uuid::new_v4().simple(), ext);
        let file_path = upload_dir.join(&unique_name);

        // 保存原始文件
        let data = match field.bytes().await {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("读取文件数据失败: {}", e)})),
                ).into_response();
            }
        };

        let mut file = match fs::File::create(&file_path).await {
            Ok(file) => file,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("创建文件失败: {}", e)})),
                ).into_response();
            }
        };

        if let Err(e) = file.write_all(&data).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("写入文件失败: {}", e)})),
            ).into_response();
        }

        // 生成缩略图
        let thumbnail_path = generate_thumbnail(&file_path, &upload_dir, &unique_name).await;

        let relative_path = format!("/uploads/products/{}/{}", product_id, unique_name);
        let thumbnail_relative_path = thumbnail_path.map(|p| format!("/uploads/products/{}/{}", product_id, p));
        
        // 记录颜色代码信息
        println!("图片: {}, 提取的颜色代码: {}", file_name, color_code);
        
        uploaded_images.push(json!({
            "url": relative_path,
            "thumbnail": thumbnail_relative_path,
            "filename": unique_name,
            "original_name": file_name,
            "size": data.len(),
            "mime_type": mime_type,
            "color_code": color_code,
        }));
        
        // 保存到数据库，包含 color_code
        let _ = sqlx::query!(
            r#"INSERT INTO product_images 
               (product_id, url, name, original_name, file_size, mime_type, color_code, sort_order) 
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            product_id,
            relative_path,
            unique_name,
            file_name,
            data.len() as i64,
            mime_type,
            if color_code.is_empty() { None } else { Some(&color_code) },
            uploaded_images.len() as i32 - 1
        )
        .execute(&state.db_pool)
        .await;
    }

    (StatusCode::OK, Json(json!({
        "success": true,
        "images": uploaded_images,
        "message": format!("成功上传 {} 张图片", uploaded_images.len())
    }))).into_response()
}

// 从文件名提取颜色代码的辅助函数
fn extract_color_code_from_filename(filename: &str) -> String {
    // 去除扩展名
    let name_without_ext = filename.split('.').next().unwrap_or(filename);
    
    // 多种命名规则处理
    let parts: Vec<&str> = name_without_ext.split('_').collect();
    
    // 如果包含下划线，取第一部分
    let code = parts.first().unwrap_or(&name_without_ext);
    
    // 去除可能的前缀
    let code = code.trim_start_matches("color-")
                   .trim_start_matches("c-");
    
    // 提取字母部分（去除数字后缀）
    // 例如：BL01 -> BL, RED123 -> RED
    let alpha_part: String = code.chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    
    let result = if !alpha_part.is_empty() {
        alpha_part.to_uppercase()
    } else {
        code.to_uppercase()
    };
    
    // 限制长度，颜色代码通常是2-4个字符
    if result.len() > 4 {
        result[..4].to_string()
    } else {
        result
    }
}

// 修复 generate_thumbnail 函数
async fn generate_thumbnail(file_path: &PathBuf, upload_dir: &PathBuf, filename: &str) -> Option<String> {
    let thumbnail_name = format!("thumb_{}", filename);
    let thumbnail_path = upload_dir.join(&thumbnail_name);
    let file_path_clone = file_path.clone();
    
    tokio::task::spawn_blocking(move || {
        let img = image::open(&file_path_clone).ok()?;
        let thumbnail = img.resize(200, 200, image::imageops::FilterType::Lanczos3);
        thumbnail.save(&thumbnail_path).ok()?;
        Some(thumbnail_name)
    }).await.ok()?
}


// 获取产品图片列表
pub async fn get_product_images_handler(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let rows = match sqlx::query!(
        "SELECT id, url, name, original_name, file_size, mime_type, sort_order, created_at 
         FROM product_images 
         WHERE product_id = $1 
         ORDER BY sort_order",
        product_id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("获取图片列表失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response();
        }
    };

    let images: Vec<_> = rows.into_iter().map(|row| {
        json!({
            "id": row.id,
            "url": row.url,
            "name": row.name,
            "original_name": row.original_name,
            "file_size": row.file_size,
            "mime_type": row.mime_type,
            "sort_order": row.sort_order,
            "created_at": row.created_at,
        })
    }).collect();

    (StatusCode::OK, Json(json!({
        "success": true,
        "images": images
    }))).into_response()
}

// 删除产品图片
pub async fn delete_product_image_handler(
    State(state): State<Arc<AppState>>,
    Path((product_id, image_id)): Path<(Uuid, i32)>,
) -> impl IntoResponse {
    // 先获取图片信息
    let row = match sqlx::query!(
        "SELECT url FROM product_images WHERE id = $1 AND product_id = $2",
        image_id, product_id
    )
    .fetch_optional(&state.db_pool)
    .await 
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "图片不存在"})),
            ).into_response();
        }
        Err(e) => {
            eprintln!("查询图片失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response();
        }
    };

    // 删除物理文件
    let file_path = PathBuf::from(&row.url.trim_start_matches('/'));
    if file_path.exists() {
        if let Err(e) = fs::remove_file(&file_path).await {
            eprintln!("删除文件失败: {}", e);
        }
        
        // 删除缩略图
        let thumbnail_path = file_path.parent().unwrap().join(format!("thumb_{}", file_path.file_name().unwrap().to_str().unwrap()));
        if thumbnail_path.exists() {
            let _ = fs::remove_file(&thumbnail_path).await;
        }
    }

    // 删除数据库记录
    match sqlx::query!("DELETE FROM product_images WHERE id = $1", image_id)
        .execute(&state.db_pool)
        .await 
    {
        Ok(_) => {
            // 重新排序剩余图片
            let _ = sqlx::query!(
                "UPDATE product_images SET sort_order = new_sort.sort_order 
                 FROM (SELECT id, ROW_NUMBER() OVER (ORDER BY sort_order) - 1 as sort_order 
                       FROM product_images WHERE product_id = $1) AS new_sort 
                 WHERE product_images.id = new_sort.id",
                product_id
            )
            .execute(&state.db_pool)
            .await;
            
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            eprintln!("删除图片记录失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

// 更新图片排序
// 修改为正确的类型定义


#[derive(Debug, Deserialize)]
pub struct ImageOrderItem {
    pub id: i32,
    pub sort_order: i32,
}

pub async fn reorder_product_images_handler(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<Vec<ImageOrderItem>>,
) -> impl IntoResponse {
    for item in payload {
        let _ = sqlx::query!(
            "UPDATE product_images SET sort_order = $1 WHERE id = $2 AND product_id = $3",
            item.sort_order, item.id, product_id
        )
        .execute(&state.db_pool)
        .await;
    }
    
    (StatusCode::OK, Json(json!({"success": true, "message": "排序已更新"}))).into_response()
}






// 获取产品的变体列表
pub async fn list_variants_handler(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.product_repo.list_variants(product_id).await {
        Ok(variants) => (StatusCode::OK, Json(variants)).into_response(),
        Err(e) => {
            eprintln!("获取变体列表失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

// 根据规则生成变体




pub async fn generate_variants_by_rule_handler(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
    Json(rule): Json<VariantGenerateRule>,
) -> impl IntoResponse {
    // 获取产品信息
    let product = match state.product_repo.get_product_by_id(product_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "产品不存在"})),
            ).into_response();
        }
        Err(e) => {
            eprintln!("获取产品失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response();
        }
    };
    
    // 获取产品的所有图片
    let images = match state.product_repo.get_product_images(product_id, None).await {
        Ok(imgs) => imgs,
        Err(e) => {
            eprintln!("获取图片失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response();
        }
    };
    
    // 获取尺码列表
  // 获取尺码列表（支持空格和逗号分隔）
let raw = match rule.size_field.as_str() {
    "csize" => product.csize.as_deref().unwrap_or(""),
    "ussize" => product.ussize.as_deref().unwrap_or(""),
    "asize" => product.asize.as_deref().unwrap_or(""),
    _ => product.csize.as_deref().unwrap_or(""),
};
let sizes: Vec<String> = raw
    .replace(',', " ")          // 将逗号替换为空格
    .split_whitespace()         // 按空白分割
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();
    //println!("解析后的尺码列表: {:?}", sizes);   // 添加调试输出
    // 获取颜色代码列表（从图片文件名提取）
    let color_codes: Vec<String> = if rule.color_code_field == "filename" {
        color_code::extract_color_codes_from_images(&images)
    } else {
        product.color_list.unwrap_or_default()
            .split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect()
    };
    
    if color_codes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "请先设置颜色代码或上传图片"})),
        ).into_response();
    }
    
    // 产品基础SKU（使用产品slug或名称）
    let product_sku = product.sku.as_ref().map(|s| s.to_uppercase()).unwrap_or_else(|| product.slug.to_uppercase());
    
    let mut generated_variants = Vec::new();
    
    // 获取现有变体，用于检查重复
    let existing_variants = match state.product_repo.list_variants(product_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("获取现有变体失败: {}", e);
            vec![]
        }
    };
    
    for color_code_str in &color_codes {
        // 使用颜色配置文件获取颜色名称
        let color_name = color_code::get_color_name(color_code_str);
        // 提取基础颜色代码用于存储（如 BL01 -> BL）
        let color_map = color_code::get_color_map(color_code_str);
        
        //println!("颜色代码: {}, 名称: {}, 分类: {}", color_code_str, color_name, color_map);
        

        for size in &sizes {
            // 根据规则生成SKU
            let sku = rule.sku_pattern
                .replace("{product_sku}", &product_sku)
                .replace("{color_code}", color_code_str)
                .replace("{size}", &size.to_uppercase());  // 转大写
            
            // 检查SKU是否已存在
            if existing_variants.iter().any(|v| v.sku == sku) {
                continue;
            }
            
            
            let variant_input = CreateVariantInput {
                product_id,
                sku: sku.clone(),
                color_code: Some(color_code_str.clone()),
                color: Some(color_map.clone()),  // 存储基础颜色代码 "BL"
                color_name: Some(color_name.clone()),      // 存储颜色名称 "Black"
                size: Some(size.clone()),
                price: rule.default_price,
                stock: rule.default_stock.unwrap_or(0),
                
            };
            
            match state.product_repo.create_variant(variant_input).await {
                Ok(variant) => {
                    generated_variants.push(variant);
                }
                Err(e) => {
                    eprintln!("创建变体失败: {}", e);
                }
            }
        }
    }
    
    (StatusCode::CREATED, Json(json!({
        "success": true,
        "generated": generated_variants.len(),
        "variants": generated_variants,
        "message": format!("成功生成 {} 个变体", generated_variants.len())
    }))).into_response()
}

// 更新变体
// 更新变体
pub async fn update_variant_handler(
    State(state): State<Arc<AppState>>,
    Path(variant_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let price = payload.get("price").and_then(|p| p.as_f64());
    let stock = payload.get("stock").and_then(|s| s.as_i64().map(|i| i as i32));
    let is_default = payload.get("is_default").and_then(|d| d.as_bool());
    
    let input = UpdateVariantInput {
        price,
        stock,
        is_default,
    };
    
    match state.product_repo.update_variant(variant_id, input).await {
        Ok(variant) => (StatusCode::OK, Json(variant)).into_response(),
        Err(e) => {
            eprintln!("更新变体失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

// 删除变体
pub async fn delete_variant_handler(
    State(state): State<Arc<AppState>>,
    Path(variant_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.product_repo.delete_variant(variant_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "变体不存在"})),
        ).into_response(),
        Err(e) => {
            eprintln!("删除变体失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}