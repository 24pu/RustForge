

pub mod auth; // 导出 auth 子模块
pub use auth::*; // 或者显式 pub use auth::admin_auth_middleware;

pub mod install;
pub mod lang;
pub use lang::lang_middleware;
pub use install::check_installed;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct CurrentUser(pub Option<uuid::Uuid>);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user_id = parts.extensions.get::<Option<String>>()
            .and_then(|opt| opt.as_ref())
            .and_then(|id| uuid::Uuid::parse_str(id).ok());
        Ok(CurrentUser(user_id))
    }
}