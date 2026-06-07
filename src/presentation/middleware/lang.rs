use axum::{
    extract::{Request, State},   // 添加 State
    http::header,
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookies;
use std::sync::Arc;
use crate::presentation::AppState;

pub async fn lang_middleware(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Response {
    let lang = cookies
        .get("lang")
        .map(|c| c.value().to_string())
        .filter(|l| state.i18n.supported_langs().contains(l))
        .unwrap_or_else(|| state.i18n.default_lang());

    req.extensions_mut().insert(lang);
    next.run(req).await
}

