use axum::{extract::Request, middleware::Next, response::Response, http::StatusCode};
use std::sync::Arc;
use crate::presentation::AppState;
use crate::presentation::middleware::CurrentUser;
use crate::core::repositories::UserRepository;

pub async fn require_permission(
    permission: &'static str,
) -> impl Fn(Request, Next) -> Response + '_ {
    move |mut req: Request, next: Next| {
        let state = req.extensions().get::<Arc<AppState>>().unwrap().clone();
        let user_id = req.extensions()
            .get::<Option<uuid::Uuid>>()
            .and_then(|opt| *opt);
        async move {
            if let Some(user_id) = user_id {
                if state.user_repo.user_has_permission(user_id, permission).await.unwrap_or(false) {
                    return next.run(req).await;
                }
            }
            (StatusCode::FORBIDDEN, "Insufficient permissions").into_response()
        }
    }
}