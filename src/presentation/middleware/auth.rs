use axum::{
    extract::{Request, State},
    http::header,   // 新增
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_cookies::Cookies;
use uuid::Uuid;
use crate::infrastructure::auth::verify_token;
use crate::core::UserRepository;
use crate::presentation::AppState;
use crate::presentation::types::UserInfo;
use std::sync::Arc;



pub async fn inject_user_info(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    // ---- 提取 auth_token Cookie ----
    let token = req
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';').find_map(|pair| {
                let mut parts = pair.trim().splitn(2, '=');
                let key = parts.next()?;
                let val = parts.next()?;
                if key == "auth_token" { Some(val.to_owned()) } else { None }
            })
        });

    // ---- 注入用户信息 ----
    let user_info = if let Some(t) = token {
        if let Ok(claims) = verify_token(&t) {
            if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                if let Ok(Some(user)) = state.user_repo.get_user_by_id(user_id).await {
                    UserInfo {
                        is_logged_in: true,
                        user_id: Some(user.id.to_string()),
                        user_name: Some(user.name.unwrap_or_else(|| user.email)),
                    }
                } else {
                    UserInfo::anonymous()
                }
            } else {
                UserInfo::anonymous()
            }
        } else {
            UserInfo::anonymous()
        }
    } else {
        UserInfo::anonymous()
    };
    req.extensions_mut().insert(user_info);

    // ---- 提取语言偏好 ----
    // 优先从 Cookie 中获取 lang，其次从 Accept-Language 头获取，最后使用默认语言
    let lang = req
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';').find_map(|pair| {
                let mut parts = pair.trim().splitn(2, '=');
                let key = parts.next()?;
                let val = parts.next()?;
                if key.trim() == "lang" { Some(val.trim().to_string()) } else { None }
            })
        })
        .filter(|l| state.i18n.supported_langs().contains(l))
        .unwrap_or_else(|| {
            // 从 Accept-Language 头获取
            req.headers()
                .get(header::ACCEPT_LANGUAGE)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.split(';').next())
                .map(|s| s.trim().to_lowercase())
                .filter(|l| state.i18n.supported_langs().contains(l))
                .unwrap_or_else(|| state.i18n.default_lang())
        });

    req.extensions_mut().insert(lang);

    // ---- 注入语言选项列表（用于模板渲染） ----
    req.extensions_mut().insert(state.i18n.lang_options());

    next.run(req).await
}




pub async fn auth_middleware(
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Response {
    // 优先从 Authorization header 获取 token

    let token = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string());
    let token = token.or_else(|| {
        cookies.get("auth_token").map(|c| c.value().to_string())
    });
    let user_id = token.and_then(|t| verify_token(&t).ok()).map(|claims| claims.sub);
    req.extensions_mut().insert(user_id);
    next.run(req).await
}

// 专门用于 /admin 的认证中间件（从 Cookie 读取 token，未登录时重定向）
pub async fn admin_auth_middleware(
    cookies: Cookies,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // 放行登录页面、静态资源、组件片段（登录页面渲染必需）
    if path.starts_with("/admin/login")
        || path.ends_with(".js")
        || path.ends_with(".css")
        || path.ends_with(".png")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
        || path.ends_with(".woff")
        || path.ends_with(".woff2")
        || path.starts_with("/admin/components/")
    {
        return next.run(req).await;
    }

    let token = cookies.get("auth_token").map(|c| c.value().to_string());
    let is_authenticated = match token {
        Some(t) => verify_token(&t).is_ok(),
        None => false,
    };

    if is_authenticated {
        next.run(req).await
    } else {
        Redirect::temporary("/admin/login.html").into_response()
    }
}