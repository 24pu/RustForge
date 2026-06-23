// infrastructure/db/product_repo.rs

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::QueryBuilder;
use crate::core::models::{Product, ProductVariant, ProductImage, ProductCategory};
use crate::core::{
    ProductRepository, CreateProductInput, UpdateProductInput, 
    CreateVariantInput, UpdateVariantInput, AddProductImageInput, 
    ColorWithName, Pagination, ProductFilters
};

pub struct PostgresProductRepo {
    pub pool: PgPool,
}

impl PostgresProductRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepo {
    async fn create_product(&self, input: CreateProductInput) -> Result<Product> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let translation_group = input.translation_group.unwrap_or_else(Uuid::new_v4);
        
        let row = sqlx::query(
            r#"INSERT INTO products (
                id,sku, slug, lang, translation_group, name, dname, fullname, brand, 
                cover_image, summary, description, keywords, points, dnote, 
                csize, ussize, asize, fabric_type, price, stock, package, weight, 
                published, user_id, size_list, color_list, color_names, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 
                      $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29,$30)
            RETURNING *"#,
        )
        .bind(id)
        .bind(&input.sku)
        .bind(&input.slug)
        .bind(input.lang.as_deref().unwrap_or("zh"))
        .bind(translation_group)
        .bind(&input.name)
        .bind(&input.dname)
        .bind(&input.fullname)
        .bind(&input.brand)
        .bind(&input.cover_image)
        .bind(&input.summary)
        .bind(&input.description)
        .bind(&input.keywords)
        .bind(&input.points)
        .bind(&input.dnote)
        .bind(&input.csize)
        .bind(&input.ussize)
        .bind(&input.asize)
        .bind(&input.fabric_type)
        .bind(&input.price)
        .bind(&input.stock)
        .bind(&input.package)
        .bind(&input.weight)
        .bind(input.published)
        .bind(input.user_id)
        .bind(&input.size_list)
        .bind(&input.color_list)
        .bind(&input.color_names)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(Product {
            id: row.get("id"),
            sku: row.get("sku"),
            slug: row.get("slug"),
            lang: row.get("lang"),
            translation_group: row.get("translation_group"),
            name: row.get("name"),
            dname: row.get("dname"),
            fullname: row.get("fullname"),
            brand: row.get("brand"),
            cover_image: row.get("cover_image"),
            summary: row.get("summary"),
            description: row.get("description"),
            keywords: row.get("keywords"),
            points: row.get("points"),
            dnote: row.get("dnote"),
            csize: row.get("csize"),
            ussize: row.get("ussize"),
            asize: row.get("asize"),
            fabric_type: row.get("fabric_type"),
            price: row.get("price"),
            stock: row.get("stock"),
            package: row.get("package"),
            weight: row.get("weight"),
            published: row.get("published"),
            user_id: row.get("user_id"),
            size_list: row.get("size_list"),
            color_list: row.get("color_list"),
            color_names: row.get("color_names"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            categories: vec![],
            variants: vec![],
            images: vec![],
        })
    }

    async fn get_product_by_slug(&self, slug: &str) -> Result<Option<Product>> {
        let row = sqlx::query!("SELECT * FROM products WHERE slug = $1", slug)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Product {
            id: r.id,
            sku: r.sku,
            slug: r.slug,
            lang: r.lang.unwrap_or_else(|| "zh".to_string()),
            translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),
            name: r.name,
            dname: r.dname,
            fullname: r.fullname,
            brand: r.brand,
            cover_image: r.cover_image,
            summary: r.summary,
            description: r.description,
            keywords: r.keywords,
            points: r.points,
            dnote: r.dnote,
            csize: r.csize,
            ussize: r.ussize,
            asize: r.asize,
            fabric_type: r.fabric_type,
            price: r.price,
            stock: r.stock,
            package: r.package,
            weight: r.weight,
            published: r.published.unwrap_or(false),
            user_id: r.user_id,
            size_list: r.size_list,
            color_list: r.color_list,
            color_names: r.color_names,
            created_at: r.created_at,
            updated_at: r.updated_at,
            categories: vec![],
            variants: vec![],
            images: vec![],
        }))
    }

    async fn get_product_by_id(&self, id: Uuid) -> Result<Option<Product>> {
        let row = sqlx::query!("SELECT * FROM products WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(r) = row {
            // 获取产品的分类
            let categories = self.get_product_categories(id).await?;
            
            Ok(Some(Product {
                id: r.id,
                sku: r.sku,
                slug: r.slug,
                lang: r.lang.unwrap_or_else(|| "zh".to_string()),  // 提供默认值
                translation_group: r.translation_group.unwrap_or_else(Uuid::new_v4),  // 提供默认值
                name: r.name,
                dname: r.dname,
                fullname: r.fullname,
                brand: r.brand,
                cover_image: r.cover_image,
                summary: r.summary,
                description: r.description,
                keywords: r.keywords,
                points: r.points,
                dnote: r.dnote,
                csize: r.csize,
                ussize: r.ussize,
                asize: r.asize,
                fabric_type: r.fabric_type,
                price: r.price,
                stock: r.stock,
                package: r.package,
                weight: r.weight,
                published: r.published.unwrap_or(false),
                user_id: r.user_id,
                size_list: r.size_list,
                color_list: r.color_list,
                color_names: r.color_names,
                created_at: r.created_at,
                updated_at: r.updated_at,
                categories,
                variants: vec![],
                images: vec![],
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_product(&self, id: Uuid, input: UpdateProductInput) -> Result<Product> {
        let now = Utc::now();
        
        let row = sqlx::query(
            r#"UPDATE products SET
                name = COALESCE($1, name),
                dname = COALESCE($2, dname),
                fullname = COALESCE($3, fullname),
                brand = COALESCE($4, brand),
                cover_image = COALESCE($5, cover_image),
                summary = COALESCE($6, summary),
                description = COALESCE($7, description),
                keywords = COALESCE($8, keywords),
                points = COALESCE($9, points),
                dnote = COALESCE($10, dnote),
                csize = COALESCE($11, csize),
                ussize = COALESCE($12, ussize),
                asize = COALESCE($13, asize),
                fabric_type = COALESCE($14, fabric_type),
                price = COALESCE($15, price),
                stock = COALESCE($16, stock),
                package = COALESCE($17, package),
                weight = COALESCE($18, weight),
                published = COALESCE($19, published),
                size_list = COALESCE($20, size_list),
                color_list = COALESCE($21, color_list),
                color_names = COALESCE($22, color_names),
                sku = COALESCE($23, sku),
                updated_at = $24
            WHERE id = $25
            RETURNING *"#,
        )
        .bind(&input.name)
        .bind(&input.dname)
        .bind(&input.fullname)
        .bind(&input.brand)
        .bind(&input.cover_image)
        .bind(&input.summary)
        .bind(&input.description)
        .bind(&input.keywords)
        .bind(&input.points)
        .bind(&input.dnote)
        .bind(&input.csize)
        .bind(&input.ussize)
        .bind(&input.asize)
        .bind(&input.fabric_type)
        .bind(&input.price)
        .bind(&input.stock)
        .bind(&input.package)
        .bind(&input.weight)
        .bind(input.published)
        .bind(&input.size_list)
        .bind(&input.color_list)
        .bind(&input.color_names)
        .bind(&input.sku)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Product {
            id: row.get("id"),
            sku: row.get("sku"),
            slug: row.get("slug"),
            lang: row.get("lang"),
            translation_group: row.get("translation_group"),
            name: row.get("name"),
            dname: row.get("dname"),
            fullname: row.get("fullname"),
            brand: row.get("brand"),
            cover_image: row.get("cover_image"),
            summary: row.get("summary"),
            description: row.get("description"),
            keywords: row.get("keywords"),
            points: row.get("points"),
            dnote: row.get("dnote"),
            csize: row.get("csize"),
            ussize: row.get("ussize"),
            asize: row.get("asize"),
            fabric_type: row.get("fabric_type"),
            price: row.get("price"),
            stock: row.get("stock"),
            package: row.get("package"),
            weight: row.get("weight"),
            published: row.get("published"),
            user_id: row.get("user_id"),
            size_list: row.get("size_list"),
            color_list: row.get("color_list"),
            color_names: row.get("color_names"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            categories: vec![],
            variants: vec![],
            images: vec![],
        })
    }

    async fn delete_product(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

async fn list_products(&self, pagination: Pagination, filters: ProductFilters) -> Result<(Vec<Product>, i64)> {
    let offset = pagination.offset() as i64;
    let limit = pagination.per_page as i64;
    
    let mut query_builder = sqlx::QueryBuilder::new(
        r#"SELECT 
            p.*,
            COALESCE(
                (SELECT json_agg(
                    json_build_object(
                        'id', pc.id,
                        'name', pc.name,
                        'slug', pc.slug
                    )
                ) 
                FROM product_category_relations pcr 
                JOIN product_categories pc ON pcr.category_id = pc.id 
                WHERE pcr.product_id = p.id),
                '[]'::json
            ) as categories_json
        FROM products p
        WHERE 1=1"#
    );
    
    // 关键词搜索
    if let Some(ref keyword) = filters.keyword {
        if !keyword.is_empty() {
            let pattern = format!("%{}%", keyword);
            query_builder.push(" AND (p.name ILIKE ");
            query_builder.push_bind(pattern.clone());
            query_builder.push(" OR p.description ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(")");
        }
    }
    
    // 状态筛选
    if let Some(published) = filters.published {
        query_builder.push(" AND p.published = ");
        query_builder.push_bind(published);
    }
    
    // 分类筛选
    if let Some(category_id) = filters.category_id {
        query_builder.push(" AND EXISTS (SELECT 1 FROM product_category_relations pcr WHERE pcr.product_id = p.id AND pcr.category_id = ");
        query_builder.push_bind(category_id);
        query_builder.push(")");
    }
    
    // 添加排序和分页
    query_builder.push(" ORDER BY p.created_at DESC");
    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);
    
    let rows = query_builder.build()
        .fetch_all(&self.pool)
        .await?;
    
    // 获取总数
    let mut count_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) as count FROM products p WHERE 1=1");
    
    if let Some(ref keyword) = filters.keyword {
        if !keyword.is_empty() {
            let pattern = format!("%{}%", keyword);
            count_builder.push(" AND (p.name ILIKE ");
            count_builder.push_bind(pattern.clone());
            count_builder.push(" OR p.description ILIKE ");
            count_builder.push_bind(pattern);
            count_builder.push(")");
        }
    }
    
    if let Some(published) = filters.published {
        count_builder.push(" AND p.published = ");
        count_builder.push_bind(published);
    }
    
    if let Some(category_id) = filters.category_id {
        count_builder.push(" AND EXISTS (SELECT 1 FROM product_category_relations pcr WHERE pcr.product_id = p.id AND pcr.category_id = ");
        count_builder.push_bind(category_id);
        count_builder.push(")");
    }
    
    let count_row = count_builder.build()
        .fetch_one(&self.pool)
        .await?;
    let total: i64 = count_row.get("count");
    
    // 解析结果
    let mut products = Vec::new();
    for row in rows {
        let categories_json: serde_json::Value = row.get("categories_json");
        let categories = if categories_json.is_array() {
            categories_json.as_array().unwrap()
                .iter()
                .filter_map(|cat| {
                    Some(ProductCategory {
                        id: cat.get("id")?.as_i64()? as i32,
                        name: cat.get("name")?.as_str()?.to_string(),
                        slug: cat.get("slug")?.as_str()?.to_string(),
                        description: None,
                        parent_id: None,
                        sort: 0,
                        show_in_nav: true,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        children: None,
                        product_count: None,  // 添加
                    })
                })
                .collect()
        } else {
            vec![]
        };
        
        products.push(Product {
            id: row.get("id"),
            slug: row.get("slug"),
            sku: row.get("sku"),
            lang: row.get("lang"),
            translation_group: row.get("translation_group"),
            name: row.get("name"),
            dname: row.get("dname"),
            fullname: row.get("fullname"),
            brand: row.get("brand"),
            cover_image: row.get("cover_image"),
            summary: row.get("summary"),
            description: row.get("description"),
            keywords: row.get("keywords"),
            points: row.get("points"),
            dnote: row.get("dnote"),
            csize: row.get("csize"),
            ussize: row.get("ussize"),
            asize: row.get("asize"),
            fabric_type: row.get("fabric_type"),
            price: row.get("price"),
            stock: row.get("stock"),
            package: row.get("package"),
            weight: row.get("weight"),
            published: row.get("published"),
            user_id: row.get("user_id"),
            size_list: row.get("size_list"),
            color_list: row.get("color_list"),
            color_names: row.get("color_names"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            categories,
            variants: vec![],
            images: vec![],
        })
    }
    
    Ok((products, total))
}

    // ========== 变体相关方法 ==========

    async fn create_variant(&self, input: CreateVariantInput) -> Result<ProductVariant> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let row = sqlx::query(
            r#"INSERT INTO product_variants 
               (id, product_id, sku, color, color_name, size, price, stock, is_default, sort_order, created_at, updated_at,color_code)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
               RETURNING id, product_id, sku, color, color_name, size, 
                         price, stock, weight, package_info, 
                         is_default, sort_order, created_at, updated_at,color_code"#
        )
        .bind(id)
        .bind(input.product_id)
        .bind(&input.sku)
        .bind(&input.color)
        .bind(&input.color_name)
        .bind(&input.size)
        .bind(input.price)
        .bind(input.stock)
        .bind(false)
        .bind(0)
        .bind(now)
        .bind(now)
        .bind(&input.color_code)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(ProductVariant {
            id: row.get("id"),
            product_id: row.get("product_id"),
            sku: row.get("sku"),
            color: row.get("color"),
            color_code: row.get("color_code"),
            color_name: row.get("color_name"),
            size: row.get("size"),
            price: row.get("price"),
            stock: row.get("stock"),
            weight: row.get("weight"),
            package_info: row.get("package_info"),
            is_default: row.get("is_default"),
            sort_order: row.get("sort_order"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn update_variant(&self, id: Uuid, input: UpdateVariantInput) -> Result<ProductVariant> {
        let now = Utc::now();
        
        let row = sqlx::query(
            r#"UPDATE product_variants SET
                price = COALESCE($1, price),
                stock = COALESCE($2, stock),
                is_default = COALESCE($3, is_default),
                updated_at = $4
            WHERE id = $5
            RETURNING id, product_id, sku, color, color_name, size, 
                      price, stock, weight, package_info, 
                      is_default, sort_order, created_at, updated_at"#
        )
        .bind(input.price)
        .bind(input.stock)
        .bind(input.is_default)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        if input.is_default == Some(true) {
            let product_id: Uuid = row.get("product_id");
            sqlx::query("UPDATE product_variants SET is_default = false WHERE product_id = $1 AND id != $2")
                .bind(product_id)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        
        Ok(ProductVariant {
            id: row.get("id"),
            product_id: row.get("product_id"),
            sku: row.get("sku"),
            color: row.get("color"),
            color_code: row.get("color_code"),
            color_name: row.get("color_name"),
            size: row.get("size"),
            price: row.get("price"),
            stock: row.get("stock"),
            weight: row.get("weight"),
            package_info: row.get("package_info"),
            is_default: row.get("is_default"),
            sort_order: row.get("sort_order"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn delete_variant(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM product_variants WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_variants(&self, product_id: Uuid) -> Result<Vec<ProductVariant>> {
        let rows = sqlx::query(
            r#"SELECT id, product_id, sku, color,color_code, color_name, size, 
                      price, stock, weight, package_info, 
                      is_default, sort_order, created_at, updated_at
               FROM product_variants 
               WHERE product_id = $1 
               ORDER BY sort_order, created_at"#
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.iter().map(|r| ProductVariant {
            id: r.get("id"),
            product_id: r.get("product_id"),
            sku: r.get("sku"),
            color: r.get("color"),
            color_code: r.get("color_code"),
            color_name: r.get("color_name"),
            size: r.get("size"),
            price: r.get("price"),
            stock: r.get("stock"),
            weight: r.get("weight"),
            package_info: r.get("package_info"),
            is_default: r.get("is_default"),
            sort_order: r.get("sort_order"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn generate_variants(&self, product_id: Uuid, colors: &[ColorWithName], sizes: &[String], default_price: Option<f64>) -> Result<Vec<ProductVariant>> {
        let mut variants = Vec::new();
        let product = self.get_product_by_id(product_id).await?;
        let product_sku = product.map(|p| p.slug.to_uppercase()).unwrap_or_default();
        let existing = self.list_variants(product_id).await?;
        let existing_skus: std::collections::HashSet<String> = existing.iter().map(|v| v.sku.clone()).collect();
        
        for color in colors {
            for size in sizes {
                let sku = format!("{}{}-{}", product_sku, color.code.to_uppercase(), size);
                if existing_skus.contains(&sku) {
                    continue;
                }
                let variant_input = CreateVariantInput {
                    product_id,
                    sku: sku.clone(),
                    color_code:  Some(color.code.clone()),
                    color: Some(color.code.clone()),
                    color_name: Some(color.name.clone()),
                    size: Some(size.clone()),
                    price: default_price,
                    stock: 0,
                };
                match self.create_variant(variant_input).await {
                    Ok(variant) => variants.push(variant),
                    Err(e) => eprintln!("创建变体失败: {}", e),
                }
            }
        }
        Ok(variants)
    }

    // ========== 图片相关方法 ==========

    // 添加产品图片
// 添加产品图片
async fn add_product_image(&self, input: AddProductImageInput) -> Result<ProductImage> {
    let now = Utc::now();
    
    // 获取当前最大排序值 - 使用 COALESCE 确保始终返回 i32，不需要 unwrap
    let max_sort: i32 = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(sort_order), -1) FROM product_images WHERE product_id = $1"
    )
    .bind(input.product_id)
    .fetch_one(&self.pool)
    .await?;  // 直接返回 i32，不需要 unwrap_or
    
    let row = sqlx::query(
        r#"INSERT INTO product_images 
           (product_id, variant_id, url, name, original_name, color_code, 
            image_type, file_size, width, height, sort_order, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
           RETURNING *"#
    )
    .bind(input.product_id)
    .bind(input.variant_id)
    .bind(&input.url)
    .bind(&input.original_name)
    .bind(&input.original_name)
    .bind(&input.color_code)
    .bind(&input.image_type)
    .bind(input.file_size)
    .bind(input.width)
    .bind(input.height)
    .bind(max_sort + 1)
    .bind(now)
    .fetch_one(&self.pool)
    .await?;
    
    Ok(ProductImage {
        id: row.get("id"),
        product_id: row.get("product_id"),
        variant_id: row.get("variant_id"),
        url: row.get("url"),
        name: row.get("name"),
        original_name: row.get("original_name"),
        color_code: row.get("color_code"),
        image_type: row.get("image_type"),
        file_size: row.get("file_size"),
        width: row.get("width"),
        height: row.get("height"),
        sort_order: row.get("sort_order"),
        created_at: row.get("created_at"),
    })
}

    async fn delete_product_image(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM product_images WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_product_images(&self, product_id: Uuid, variant_id: Option<Uuid>) -> Result<Vec<ProductImage>> {
        let rows = if let Some(vid) = variant_id {
            sqlx::query(
                "SELECT * FROM product_images WHERE product_id = $1 AND (variant_id = $2 OR variant_id IS NULL) ORDER BY sort_order"
            )
            .bind(product_id)
            .bind(vid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT * FROM product_images WHERE product_id = $1 ORDER BY sort_order"
            )
            .bind(product_id)
            .fetch_all(&self.pool)
            .await?
        };
        
        Ok(rows.iter().map(|r| ProductImage {
            id: r.get("id"),
            product_id: r.get("product_id"),
            variant_id: r.get("variant_id"),
            url: r.get("url"),
            name: r.get("name"),
            original_name: r.get("original_name"),
            color_code: r.get("color_code"),
            image_type: r.get("image_type"),
            file_size: r.get("file_size"),
            width: r.get("width"),
            height: r.get("height"),
            sort_order: r.get("sort_order"),
            created_at: r.get("created_at"),
        }).collect())
    }

    // ========== 分类相关方法 ==========

    async fn set_product_categories(&self, product_id: Uuid, category_ids: &[i32]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        
        sqlx::query!("DELETE FROM product_category_relations WHERE product_id = $1", product_id)
            .execute(&mut *tx)
            .await?;
        
        for &category_id in category_ids {
            sqlx::query!(
                "INSERT INTO product_category_relations (product_id, category_id) VALUES ($1, $2)",
                product_id, category_id
            )
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }

    async fn get_product_categories(&self, product_id: Uuid) -> Result<Vec<ProductCategory>> {
        let rows = sqlx::query!(
            r#"SELECT pc.id, pc.name, pc.slug, pc.description, pc.parent_id, pc.sort, pc.show_in_nav, pc.created_at, pc.updated_at
            FROM product_categories pc
            INNER JOIN product_category_relations pcr ON pc.id = pcr.category_id
            WHERE pcr.product_id = $1
            ORDER BY pc.sort"#,
            product_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| ProductCategory {
            id: r.id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            parent_id: r.parent_id,
            sort: r.sort.unwrap_or(0),
            show_in_nav: r.show_in_nav.unwrap_or(true),
            created_at: r.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: r.updated_at.unwrap_or_else(chrono::Utc::now),
            children: None,
            product_count: None,  // 添加
        }).collect())
    }
}