use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use crate::presentation::AppState;
use sqlx::PgPool;   // 添加

/// 获取语言配置（支持的语言列表 + 默认语言 + 所有可用语言包）
pub async fn get_lang_settings_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let available = state.i18n.lang_options();
    let supported = state.i18n.supported_langs();
    let default_lang = state.i18n.default_lang();

    (StatusCode::OK, Json(json!({
        "available_langs": available,          // 所有可用的语言包
        "supported_langs": supported,          // 当前支持的语言
        "default_lang": default_lang,          // 默认语言
    }))).into_response()
}

/// 更新语言配置
pub async fn update_lang_settings_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let supported_langs: Vec<String> = payload.get("supported_langs")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let default_lang = payload.get("default_lang")
        .and_then(|v| v.as_str())
        .unwrap_or("zh")
        .to_string();

    // 保存到数据库
    if let Err(e) = save_lang_config(&state.db_pool, &supported_langs, &default_lang).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
    }

    // 重载语言管理器
    state.i18n.reload(supported_langs.clone(), default_lang.clone());

    (StatusCode::OK, Json(json!({ "message": "Language settings updated" }))).into_response()
}

async fn save_lang_config(pool: &PgPool, langs: &[String], default_lang: &str) -> Result<(), anyhow::Error> {
    let langs_str = langs.join(",");
    sqlx::query!(
        "INSERT INTO site_config (key, value) VALUES ('supported_langs', $1), ('default_lang', $2)
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        langs_str, default_lang
    )
    .execute(pool)
    .await?;
    Ok(())
}