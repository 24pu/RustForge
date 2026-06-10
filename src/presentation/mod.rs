//RustForge mod.rs


mod middleware;
mod handlers;
mod types;
use sqlx::PgPool;  // 添加 PgPool 导入
use crate::presentation::handlers::lang_settings::{get_lang_settings_handler, update_lang_settings_handler};
use crate::presentation::middleware::lang_middleware;
use axum::{
    Router, routing::{get, post, put, delete},
    extract::State,
    http::StatusCode,
    response::{IntoResponse,Redirect}
};
use crate::infrastructure::i18n::I18n;
use tower_http::services::ServeDir;
use tower_cookies::CookieManagerLayer;
use std::sync::Arc;
use std::net::SocketAddr;

use crate::presentation::handlers::install_handler;
use crate::presentation::middleware::check_installed;
use crate::infrastructure::config::Config;
use crate::infrastructure::theme::manager::TeraThemeManager;
use crate::infrastructure::db::create_pool;
use crate::infrastructure::db::{PostgresUserRepo, PostgresContentRepo, PostgresMediaRepo, PostgresMediaFolderRepo,PostgresPluginRepo,PostgresPluginSettingsRepo};
use crate::core::{UserRepository, ContentRepository, MediaRepository, MediaFolderRepository,PluginRepository,PluginSettingsRepository};
use crate::presentation::middleware::auth_middleware;
use crate::presentation::handlers::*;
use crate::presentation::middleware::admin_auth_middleware;
use crate::presentation::middleware::inject_user_info;
use crate::presentation::handlers::sitemap::sitemap_handler;
use crate::presentation::handlers::plugin_settings::{get_plugin_settings, update_plugin_settings,plugin_settings_view_handler,get_public_plugin_settings};
use crate::presentation::handlers::auth::logout_handler;
use crate::presentation::handlers::plugin_admin::scan_available_plugins_handler;
use crate::presentation::handlers::server_status::server_status_handler;
use crate::presentation::search::search_page_handler;

pub struct AppState {
    pub theme_manager: Arc<tokio::sync::RwLock<TeraThemeManager>>,
    pub user_repo: Arc<dyn UserRepository>,
    pub content_repo: Arc<dyn ContentRepository>,
    pub media_repo: Arc<dyn MediaRepository>,
    pub media_folder_repo: Arc<dyn MediaFolderRepository>,
    pub db_pool: sqlx::PgPool,
    pub config: Config,
    pub plugin_repo: Arc<dyn PluginRepository>,
    pub plugin_settings_repo: Arc<dyn PluginSettingsRepository>,
    pub i18n: Arc<I18n>,
}

// 辅助函数
async fn get_config_value_from_db(pool: &PgPool, key: &str) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        "SELECT value FROM site_config WHERE key = $1",
        key
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.value).unwrap_or_default())
}


pub async fn run(config: Config) -> anyhow::Result<()> {
    let pool = create_pool(&config.database.url, config.database.max_connections).await?;
    let user_repo = Arc::new(PostgresUserRepo::new(pool.clone()));
    let content_repo = Arc::new(PostgresContentRepo::new(pool.clone()));
    let media_repo = Arc::new(PostgresMediaRepo::new(pool.clone()));
    let media_folder_repo = Arc::new(PostgresMediaFolderRepo::new(pool.clone()));

    let plugin_repo = Arc::new(PostgresPluginRepo::new(pool.clone()));
    let plugin_settings_repo = Arc::new(PostgresPluginSettingsRepo::new(pool.clone()));

    // ---- 初始化 i18n（支持插件语言包） ----
    let i18n = Arc::new(I18n::new("locales", "plugins"));

    // 从数据库读取语言配置
    let supported_langs_str = get_config_value_from_db(&pool, "supported_langs").await
        .unwrap_or_else(|_| "zh,en".to_string());
    let default_lang = get_config_value_from_db(&pool, "default_lang").await
        .unwrap_or_else(|_| "zh".to_string());
    let supported_langs: Vec<String> = supported_langs_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // 加载已启用插件的语言包
    i18n.reload_with_plugins(supported_langs, default_lang, &pool).await;

    // ---- 初始化主题管理器 ----
    let theme_manager = Arc::new(tokio::sync::RwLock::new(
        TeraThemeManager::scan_and_load(&config.theme.themes_dir, i18n.clone()).await?
    ));

    // ---- 创建 state ----
    let state = Arc::new(AppState {
        theme_manager,
        user_repo,
        content_repo,
        media_repo,
        media_folder_repo,
        db_pool: pool.clone(),
        config: config.clone(),
        plugin_repo,
        plugin_settings_repo,
        i18n: i18n.clone(),
    });

    let protected_api = Router::new()
        .route("/api/me", get(me_handler))
        .route("/api/admin/contents", get(list_contents_handler))
        .route("/api/admin/contents", post(create_content_handler))
        .route("/api/admin/contents/:id", put(update_content_handler))
        .route("/api/admin/contents/:id", delete(delete_content_handler))
        .route("/api/admin/users", get(list_users_handler))
        .route("/api/admin/users/:id/roles", put(update_user_roles_handler))
        .route("/api/me/permissions", get(my_permissions_handler))
        .route("/api/me/password", put(change_password_handler))
        .route("/api/admin/contents/:id", get(get_content_by_id_handler))
        .route("/api/admin/categories/tree", get(list_categories_tree_handler))
        .route("/api/admin/categories", post(create_category_handler))
        .route("/api/admin/categories/:id", put(update_category_handler))
        .route("/api/admin/categories/:id", delete(delete_category_handler))
        .route("/api/admin/categories/:id", get(get_category_by_id_handler))
        .route("/api/admin/users/:id", delete(delete_user_handler))
        .route("/api/admin/categories/reorder", post(reorder_categories_handler))
        .route("/api/admin/media", get(list_media_handler))
        .route("/api/admin/media", post(upload_media_handler))
        .route("/api/admin/media/:id", delete(delete_media_handler))
        .route("/api/admin/media/folders/tree", get(list_folders_tree_handler))
        .route("/api/admin/media/folders", post(create_folder_handler))
        .route("/api/admin/media/folders/:id", put(rename_folder_handler))
        .route("/api/admin/media/folders/:id", delete(delete_folder_handler))
        .route("/api/admin/roles", get(list_roles_handler))
        .route("/api/admin/roles", post(create_role_handler))
        .route("/api/admin/roles/:id", put(update_role_handler))
        .route("/api/admin/roles/:id", delete(delete_role_handler))
        .route("/api/admin/permissions", get(list_permissions_handler))
        .route("/api/admin/roles/:id/permissions", get(get_role_permissions_handler))
        .route("/api/admin/roles/:id/permissions", put(update_role_permissions_handler))
        .route("/api/admin/config", get(get_config_handler))
        .route("/api/admin/config", put(update_config_handler))
        .route("/api/admin/themes", get(list_themes_handler))
        .route("/api/admin/themes/activate", post(activate_theme_handler))
        .route("/api/admin/themes/:name/templates", get(list_templates_handler))
        .route("/api/admin/themes/:name/templates/:filename", get(get_template_handler))
        .route("/api/admin/themes/:name/templates/:filename", put(update_template_handler))
        .route("/api/admin/plugins", get(list_plugins_handler))
        .route("/api/admin/plugins", post(install_plugin_handler))
        .route("/api/admin/plugins/:id", put(toggle_plugin_handler))
        .route("/api/admin/plugins/:id", delete(uninstall_plugin_handler))
        .route("/api/plugin/:name", post(plugin_gateway_handler).get(plugin_gateway_handler))
        .route("/api/admin/plugins/available", get(scan_available_plugins_handler))
        .route("/api/admin/lang-settings", get(get_lang_settings_handler).put(update_lang_settings_handler))
        .route("/api/admin/plugins/:name/settings", get(get_plugin_settings).put(update_plugin_settings))
        .layer(axum::middleware::from_fn(auth_middleware));

    let admin_assets = ServeDir::new("frontend/dist/admin");
    let public_assets = ServeDir::new("frontend/dist").append_index_html_on_directories(true);

    let admin_router = Router::new()
        .route("/admin/login", get(|| async { Redirect::temporary("/admin/login.html") }))
        .nest_service("/admin", admin_assets)
        .layer(axum::middleware::from_fn(admin_auth_middleware));

    let plugin_settings_router = Router::new()
        .route("/plugins/:plugin_name/settings", get(plugin_settings_view_handler))
        .layer(axum::middleware::from_fn(admin_auth_middleware));

    let app = Router::new()
        .route("/", get(home_handler))
        .route("/search", get(search_page_handler))
        .route("/api/server-status", get(server_status_handler))
        .route("/health", get(health_handler))
        .route("/api/public/plugin/:name/settings", get(get_public_plugin_settings))
        .route("/api/register", post(register_handler))
        .route("/api/login", post(login_handler))
        .route("/api/logout", post(logout_handler))
        .route("/api/contents", get(list_published_contents_handler))
        .route("/sitemap.xml", get(sitemap_handler))
        .route("/install", get(|| async { 
            axum::response::Html(include_str!("../../frontend/dist/install.html"))
        }))
        .route("/api/install", post(install_handler))
        .merge(protected_api)
        .merge(admin_router)
        .merge(plugin_settings_router)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .route("/content/:slug", get(content_detail_handler))
        .route("/:slug", get(category_page_handler))
        .route("/plugins/:plugin_name/static/*file", get(plugin_static_handler))
        .route("/plugins/:plugin_name/:page", get(plugin_page_handler))
        .layer(axum::middleware::from_fn_with_state(state.clone(), lang_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), check_installed))
        .layer(CookieManagerLayer::new())
        .layer(axum::middleware::from_fn_with_state(state.clone(), inject_user_info))
        .with_state(state);

    let addr_str = format!("{}:{}", config.server.host, config.server.port);
    let socket_addr: SocketAddr = addr_str.parse()?;
    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}