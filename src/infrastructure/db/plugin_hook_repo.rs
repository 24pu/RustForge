use sqlx::{PgPool, QueryBuilder};
use anyhow::Result;
use async_trait::async_trait;
use crate::core::models::{PluginHook,CreatePluginHookRequest,UpdatePluginHookRequest};
use crate::core::PluginHookRepository;



pub struct PostgresPluginHookRepo {
    pool: PgPool,
}

impl PostgresPluginHookRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PluginHookRepository for PostgresPluginHookRepo {
    async fn list_by_hook(
        &self,
        hook_name: &str,
        lang: &str,
        enabled: Option<bool>,
    ) -> Result<Vec<PluginHook>> {
        let mut qb = QueryBuilder::new(
            "SELECT id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at
            FROM plugin_hooks WHERE hook_name = "
        );
        qb.push_bind(hook_name);
        // 如果 lang 不是空字符串且不是 "all"，则添加语言过滤
        if !lang.is_empty() && lang != "all" {
            qb.push(" AND lang = ");
            qb.push_bind(lang);
        }
        if let Some(e) = enabled {
            qb.push(" AND enabled = ");
            qb.push_bind(e);
        }
        qb.push(" ORDER BY sort_order, id");
        let rows = qb.build_query_as::<PluginHook>().fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<PluginHook>> {
        let row = sqlx::query_as::<_, PluginHook>(
            "SELECT id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at
             FROM plugin_hooks WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create(&self, req: &CreatePluginHookRequest) -> Result<PluginHook> {
        let row = sqlx::query_as::<_, PluginHook>(
            "INSERT INTO plugin_hooks (plugin_name, hook_name, content, sort_order, lang, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at"
        )
        .bind(&req.plugin_name)
        .bind(&req.hook_name)
        .bind(&req.content)
        .bind(req.sort_order)
        .bind(req.lang.as_deref().unwrap_or("zh"))
        .bind(req.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

   async fn update(&self, id: i32, req: &UpdatePluginHookRequest) -> Result<PluginHook> {
        let row = sqlx::query_as::<_, PluginHook>(
            "UPDATE plugin_hooks SET
                content = COALESCE($1, content),
                sort_order = COALESCE($2, sort_order),
                enabled = COALESCE($3, enabled),
                lang = COALESCE($4, lang),
                updated_at = now()
            WHERE id = $5
            RETURNING id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at"
        )
        .bind(&req.content)
        .bind(req.sort_order)
        .bind(req.enabled)
        .bind(&req.lang)      // 绑定 lang
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn delete(&self, id: i32) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM plugin_hooks WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    async fn list_all_generic(&self, enabled: Option<bool>) -> Result<Vec<PluginHook>> {
        let mut qb = QueryBuilder::new(
            "SELECT id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at
            FROM plugin_hooks WHERE lang = ''"
        );
        if let Some(e) = enabled {
            qb.push(" AND enabled = ");
            qb.push_bind(e);
        }
        qb.push(" ORDER BY hook_name, sort_order, id");
        let rows = qb.build_query_as::<PluginHook>().fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn list_by_plugin(&self, plugin_name: &str, enabled: Option<bool>) -> Result<Vec<PluginHook>> {
        let mut qb = QueryBuilder::new(
            "SELECT id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at
            FROM plugin_hooks WHERE plugin_name = "
        );
        qb.push_bind(plugin_name);
        if let Some(e) = enabled {
            qb.push(" AND enabled = ");
            qb.push_bind(e);
        }
        qb.push(" ORDER BY hook_name, sort_order, id");
        let rows = qb.build_query_as::<PluginHook>().fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn list_by_lang(&self, lang: &str, enabled: Option<bool>) -> Result<Vec<PluginHook>> {
        let mut qb = QueryBuilder::new(
            "SELECT id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at
            FROM plugin_hooks WHERE lang = "
        );
        qb.push_bind(lang);
        if let Some(e) = enabled {
            qb.push(" AND enabled = ");
            qb.push_bind(e);
        }
        qb.push(" ORDER BY hook_name, sort_order, id");
        let rows = qb.build_query_as::<PluginHook>().fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn list_enabled_by_lang(&self, lang: &str) -> Result<Vec<PluginHook>> {
        // 注意：使用 query_as! 需显式指定字段，但已有 PluginHook 结构体
        let rows = sqlx::query_as!(
            PluginHook,
            r#"
            SELECT ph.id, ph.plugin_name, ph.hook_name, ph.content, ph.sort_order, ph.lang, ph.enabled, ph.created_at, ph.updated_at
            FROM plugin_hooks ph
            INNER JOIN plugins p ON p.name = ph.plugin_name
            WHERE ph.lang = $1 AND ph.enabled = true AND p.enabled = true
            ORDER BY ph.hook_name, ph.sort_order, ph.id
            "#,
            lang
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// 获取所有钩子（全局，不分插件），按创建时间降序，支持分页
     async fn list_all_hooks(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<PluginHook>, i64)> {
        let offset = (page - 1) * per_page;

        let count_sql = "SELECT COUNT(*) FROM plugin_hooks";
        let total: i64 = sqlx::query_scalar(count_sql)
            .fetch_one(&self.pool)
            .await?;

        let rows = sqlx::query!(
            r#"
            SELECT 
                id, plugin_name, hook_name, content, sort_order, lang, enabled, created_at, updated_at
            FROM plugin_hooks
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            per_page,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut hooks = Vec::new();
        for row in rows {
            hooks.push(PluginHook {
                id: row.id,
                plugin_name: row.plugin_name,
                hook_name: row.hook_name,
                content: row.content,
                sort_order: row.sort_order,
                lang: row.lang,
                enabled: row.enabled,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }
        Ok((hooks, total))
    }
}