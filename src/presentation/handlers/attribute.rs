// src/presentation/handlers/attribute.rs

use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result as AnyResult;

use crate::presentation::AppState;
use crate::presentation::middleware::CurrentUser;
use crate::presentation::handlers::utils::check_permission;
use crate::core::{
    AttributeTemplateRepository, AttributeGroupRepository, ProductAttributeValueRepository,
    CreateAttributeTemplateInput, UpdateAttributeTemplateInput,
    CreateAttributeGroupInput, UpdateAttributeGroupInput,
    ProductAttributeValueInput,
};
use crate::core::models::{AttributeTemplate, AttributeGroup, ProductAttributeValue};
use crate::infrastructure::db::{
    PostgresAttributeTemplateRepo,
    PostgresAttributeGroupRepo,
    PostgresProductAttributeValueRepo,
};

// ---------- Request/Response DTOs ----------

#[derive(Debug, Deserialize)]
pub struct ListTemplateParams {
    pub is_used: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub title: Option<String>,
    pub value: Option<String>,
    pub is_used: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub title: Option<String>,
    pub value: Option<String>,
    pub is_used: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_used: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_used: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AddTemplateToGroupRequest {
    pub template_id: i32,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSortRequest {
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct SetAttributeValuesRequest {
    pub values: Vec<ProductAttributeValueInput>,
}

// ---------- Error Handling ----------

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden").into_response(),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            AppError::Internal(e) => {
                eprintln!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}

// Helper to convert permission check to AppError
async fn require_permission(user_id: Uuid, state: &AppState, perm: &str) -> Result<(), AppError> {
    check_permission(Some(user_id), &state.user_repo, perm)
        .await
        .map_err(|(status, msg)| match status {
            StatusCode::UNAUTHORIZED => AppError::Unauthorized,
            StatusCode::FORBIDDEN => AppError::Forbidden,
            _ => AppError::Internal(anyhow::anyhow!(msg)),
        })
}

// ---------- Attribute Template Handlers ----------

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListTemplateParams>,
) -> Result<Json<Vec<AttributeTemplate>>, AppError> {
    let repo = PostgresAttributeTemplateRepo::new(state.db_pool.clone());
    let templates = repo.list_templates(params.is_used).await?;
    Ok(Json(templates))
}

pub async fn create_template(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<Json<AttributeTemplate>, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:create").await?;

    let repo = PostgresAttributeTemplateRepo::new(state.db_pool.clone());
    let input = CreateAttributeTemplateInput {
        name: payload.name,
        title: payload.title,
        value: payload.value,
        is_used: payload.is_used,
        user_id: Some(user_id),
    };
    let template = repo.create_template(input).await?;
    Ok(Json(template))
}

pub async fn update_template(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<Json<AttributeTemplate>, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:edit").await?;

    let repo = PostgresAttributeTemplateRepo::new(state.db_pool.clone());
    let input = UpdateAttributeTemplateInput {
        name: payload.name,
        title: payload.title,
        value: payload.value,
        is_used: payload.is_used,
    };
    let template = repo.update_template(id, input).await?;
    Ok(Json(template))
}

pub async fn delete_template(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:delete").await?;

    let repo = PostgresAttributeTemplateRepo::new(state.db_pool.clone());
    let deleted = repo.delete_template(id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("模板不存在".to_string()))
    }
}

// ---------- Attribute Group Handlers ----------

pub async fn list_groups(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AttributeGroup>>, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:list").await?;

    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    let groups = repo.list_groups(Some(user_id)).await?;
    Ok(Json(groups))
}

pub async fn create_group(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Json<AttributeGroup>, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:create").await?;

    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    let input = CreateAttributeGroupInput {
        name: payload.name,
        description: payload.description,
        user_id: Some(user_id),
        is_used: payload.is_used,
        sort_order: payload.sort_order,
    };
    let group = repo.create_group(input).await?;
    Ok(Json(group))
}

pub async fn update_group(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<Json<AttributeGroup>, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:edit").await?;

    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    let input = UpdateAttributeGroupInput {
        name: payload.name,
        description: payload.description,
        is_used: payload.is_used,
        sort_order: payload.sort_order,
    };
    let group = repo.update_group(id, input).await?;
    Ok(Json(group))
}

pub async fn delete_group(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:delete").await?;

    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    let deleted = repo.delete_group(id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("分组不存在".to_string()))
    }
}

// ---------- Group-Template Relations ----------

pub async fn get_group_templates(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i32>,
) -> Result<Json<Vec<AttributeTemplate>>, AppError> {
    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    let templates = repo.get_group_templates(group_id).await?;
    Ok(Json(templates))
}

pub async fn add_template_to_group(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i32>,
    Json(payload): Json<AddTemplateToGroupRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:edit").await?;

    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    repo.add_template_to_group(
        group_id,
        payload.template_id,
        payload.sort_order.unwrap_or(0),
    )
    .await?;
    Ok(StatusCode::CREATED)
}

pub async fn remove_template_from_group(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path((group_id, template_id)): Path<(i32, i32)>,
) -> Result<StatusCode, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:edit").await?;

    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    repo.remove_template_from_group(group_id, template_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_template_sort(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path((group_id, template_id)): Path<(i32, i32)>,
    Json(payload): Json<UpdateSortRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:edit").await?;

    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    repo.update_template_sort_in_group(group_id, template_id, payload.sort_order)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------- Product Attribute Values ----------

pub async fn get_product_attribute_values(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<ProductAttributeValue>>, AppError> {
    let repo = PostgresProductAttributeValueRepo::new(state.db_pool.clone());
    let values = repo.get_product_attribute_values(product_id).await?;
    Ok(Json(values))
}

pub async fn set_product_attribute_values(
    CurrentUser(user_opt): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<SetAttributeValuesRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = user_opt.ok_or(AppError::Unauthorized)?;
    require_permission(user_id, &state, "template:edit").await?;

    let repo = PostgresProductAttributeValueRepo::new(state.db_pool.clone());
    repo.set_product_attribute_values(product_id, &payload.values).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<AttributeTemplate>, AppError> {
    let repo = PostgresAttributeTemplateRepo::new(state.db_pool.clone());
    let template = repo.get_template_by_id(id).await?
        .ok_or_else(|| AppError::NotFound("模板不存在".to_string()))?;
    Ok(Json(template))
}

pub async fn get_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<AttributeGroup>, AppError> {
    let repo = PostgresAttributeGroupRepo::new(state.db_pool.clone());
    let group = repo.get_group_by_id(id).await?
        .ok_or_else(|| AppError::NotFound("分组不存在".to_string()))?;
    Ok(Json(group))
}