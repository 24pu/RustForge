use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{Redirect, Response, IntoResponse},
};
use std::sync::Arc;
use crate::presentation::AppState;

pub async fn check_installed(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    //eprintln!(">>> check_installed: path={}", req.uri().path());  // 调试日志

    if req.uri().path().starts_with("/install") || req.uri().path() == "/api/install" {
        return next.run(req).await;
    }

    let installed = sqlx::query("SELECT 1 FROM install_lock LIMIT 1")
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten()
        .is_some();

    if installed {
        next.run(req).await
    } else {
       // eprintln!(">>> Not installed, redirecting to /install");
        Redirect::temporary("/install").into_response()
    }
}