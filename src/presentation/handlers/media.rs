use axum::{extract::{State, Path, Query, Multipart}, Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;
use std::path::Path as StdPath;
use uuid::Uuid;
use chrono::Utc;
use std::fs;

use crate::presentation::AppState;
use crate::presentation::types::*;
use crate::presentation::handlers::utils::{check_permission, get_config_value, generate_thumbnail};
use crate::core::models::MediaFile;

use crate::presentation::middleware::CurrentUser;

pub async fn list_media_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListMediaParams>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    let has_perm = state.user_repo.user_has_permission(user_id, "media:list").await.unwrap_or(false);
    if !has_perm {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let folder_id = params.folder_id;

    let total = match state.media_repo.count_media_by_folder(folder_id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to count media: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch count").into_response();
        }
    };
    let items: Vec<MediaFile> = match state.media_repo.list_media_by_folder(folder_id, per_page as i64, offset as i64).await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Failed to list media: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch media").into_response();
        }
    };
    let total_pages = (total + per_page as i64 - 1) / per_page as i64;
    let resp = json!({
        "items": items,
        "total": total,
        "total_pages": total_pages,
        "current_page": page,
    });
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn upload_media_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    let has_perm = state.user_repo.user_has_permission(user_id, "media:upload").await.unwrap_or(false);
    if !has_perm {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    let allowed_types_str = match get_config_value(&state.db_pool, "allowed_file_types").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get config").into_response();
        }
    };
    let allowed_extensions: Vec<&str> = allowed_types_str.split(',').map(|s| s.trim()).collect();
    let max_size_mb: usize = match get_config_value(&state.db_pool, "max_file_size_mb").await {
        Ok(s) => s.parse().unwrap_or(10),
        Err(e) => {
            eprintln!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get config").into_response();
        }
    };
    let max_size_bytes = max_size_mb * 1024 * 1024;

    let upload_dir = StdPath::new("uploads");
    if !upload_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(upload_dir) {
            eprintln!("Failed to create uploads directory: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create upload directory").into_response();
        }
    }

    let mut folder_id: Option<i32> = None;
    let mut files = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");
        if name == "folder_id" {
            let value = match field.text().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to read folder_id: {}", e);
                    continue;
                }
            };
            folder_id = if value.is_empty() { None } else { value.parse().ok() };
            println!("📁 Parsed folder_id: {:?}", folder_id);
        } else if name == "files" {
            let file_name = match field.file_name() {
                Some(name) => name.to_string(),
                None => continue,
            };
            let extension = StdPath::new(&file_name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            if !allowed_extensions.contains(&extension.as_str()) {
                return (StatusCode::BAD_REQUEST, format!("File type .{} not allowed", extension)).into_response();
            }
            let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let data = match field.bytes().await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to read file data: {}", e);
                    return (StatusCode::BAD_REQUEST, "Failed to read file").into_response();
                }
            };
            if data.len() > max_size_bytes {
                return (StatusCode::BAD_REQUEST, format!("File too large (max {}MB)", max_size_mb)).into_response();
            }
            files.push((file_name, extension, mime_type, data));
        }
    }

    let mut uploaded_files = Vec::new();
    for (file_name, extension, mime_type, data) in files {
        let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);
        let storage_path = upload_dir.join(&unique_name);
        if let Err(e) = std::fs::write(&storage_path, &data) {
            eprintln!("Failed to save file: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file").into_response();
        }

        let mut thumbnail_path = None;
        if mime_type.starts_with("image/") {
            let thumb_dir = upload_dir.join("thumbnails");
            if let Err(e) = std::fs::create_dir_all(&thumb_dir) {
                eprintln!("Failed to create thumbnails directory: {}", e);
            } else {
                let thumb_name = format!("thumb_{}", unique_name);
                let thumb_full_path = thumb_dir.join(&thumb_name);
                if let Err(e) = generate_thumbnail(&storage_path, &thumb_full_path, 200) {
                    eprintln!("Failed to generate thumbnail: {}", e);
                } else {
                    thumbnail_path = Some(format!("uploads/thumbnails/{}", thumb_name));
                }
            }
        }

        let thumb_path_for_delete = thumbnail_path.clone();
        let media = MediaFile {
            id: 0,
            filename: file_name,
            storage_path: storage_path.to_string_lossy().to_string(),
            file_size: data.len() as i64,
            mime_type,
            extension,
            folder_id,
            thumbnail_path,
            uploaded_by: Some(user_id),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        match state.media_repo.create_media(&media).await {
            Ok(m) => uploaded_files.push(m),
            Err(e) => {
                eprintln!("Failed to save media record: {}", e);
                let _ = std::fs::remove_file(&storage_path);
                if let Some(thumb) = thumb_path_for_delete {
                    let _ = std::fs::remove_file(StdPath::new(&thumb));
                }
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save record").into_response();
            }
        }
    }
    (StatusCode::CREATED, Json(uploaded_files)).into_response()
}

pub async fn delete_media_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    let has_perm = state.user_repo.user_has_permission(user_id, "media:delete").await.unwrap_or(false);
    if !has_perm {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    let media = match state.media_repo.get_media_by_id(id).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "Media not found").into_response(),
        Err(e) => {
            eprintln!("Failed to get media: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };
    if let Err(e) = fs::remove_file(&media.storage_path) {
        eprintln!("Failed to delete file {}: {}", media.storage_path, e);
    }
    match state.media_repo.delete_media(id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Media not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete media").into_response(),
    }
}

pub async fn list_folders_tree_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "folder:list").await {
        return (status, msg).into_response();
    }
    match state.media_folder_repo.list_folders_tree(None).await {
        Ok(tree) => (StatusCode::OK, Json(tree)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch folders").into_response(),
    }
}

pub async fn create_folder_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateFolderRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "folder:create").await {
        return (status, msg).into_response();
    }
    match state.media_folder_repo.create_folder(&payload.name, payload.parent_id, user_id).await {
        Ok(folder) => (StatusCode::CREATED, Json(folder)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create folder: {}", e)).into_response(),
    }
}

pub async fn rename_folder_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<RenameFolderRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "folder:edit").await {
        return (status, msg).into_response();
    }
    match state.media_folder_repo.update_folder(id, &payload.name).await {
        Ok(folder) => (StatusCode::OK, Json(folder)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to rename folder: {}", e)).into_response(),
    }
}

pub async fn delete_folder_handler(
    CurrentUser(user_id): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_permission(user_id, &state.user_repo, "folder:delete").await {
        return (status, msg).into_response();
    }
    let file_count = state.media_repo.count_media_by_folder(Some(id)).await.unwrap_or(0);
    if file_count > 0 {
        return (StatusCode::BAD_REQUEST, "Folder is not empty").into_response();
    }
    match state.media_folder_repo.delete_folder(id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Folder not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete folder").into_response(),
    }
}