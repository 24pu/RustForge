use axum::{extract::State, http::{header, StatusCode}, response::IntoResponse};
use std::sync::Arc;
use crate::presentation::AppState;

pub async fn sitemap_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let entries = match state.content_repo.list_all_published().await {
        Ok(list) => list,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch contents").into_response(),
    };

    let category_slugs = match state.content_repo.get_all_public_category_slugs().await {
        Ok(slugs) => slugs,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch categories").into_response(),
    };

    let base_url = "http://localhost:3000"; // 建议从 state.config.site.base_url 读取

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

    xml.push_str(&format!(
        r#"<url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>"#,
        base_url
    ));

    for slug in &category_slugs {
        xml.push_str(&format!(
            r#"<url><loc>{}/{}</loc><changefreq>weekly</changefreq><priority>0.9</priority></url>"#,
            base_url, slug
        ));
    }

    for entry in &entries {
        let lastmod = entry.updated_at.format("%Y-%m-%d").to_string();
        xml.push_str(&format!(
            r#"<url><loc>{}/content/{}</loc><lastmod>{}</lastmod><changefreq>weekly</changefreq><priority>0.8</priority></url>"#,
            base_url, entry.slug, lastmod
        ));
    }

    xml.push_str("</urlset>");

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    ).into_response()
}