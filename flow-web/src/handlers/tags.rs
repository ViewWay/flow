use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_domain::content::Tag;
use flow_api::extension::ListOptions;
use crate::AppState;
use serde::Serialize;

/// Tag列表响应
#[derive(Debug, Serialize)]
pub struct TagListResponse {
    pub items: Vec<Tag>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 创建Tag
/// POST /api/v1alpha1/tags
pub async fn create_tag(
    State(state): State<AppState>,
    Json(tag): Json<Tag>,
) -> Result<Response, StatusCode> {
    match state.tag_service.create(tag).await {
        Ok(tag) => Ok(Json(tag).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取Tag
/// GET /api/v1alpha1/tags/{name}
pub async fn get_tag(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.tag_service.get(&name).await {
        Ok(Some(tag)) => Ok(Json(tag).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出Tags
/// GET /api/v1alpha1/tags
pub async fn list_tags(
    State(state): State<AppState>,
    Query(params): Query<ListOptions>,
) -> Result<Response, StatusCode> {
    match state.tag_service.list(params).await {
        Ok(result) => {
            let response = TagListResponse {
                items: result.items,
                total: result.total,
                page: result.page as u64,
                size: result.size as u64,
            };
            Ok(Json(response).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 更新Tag
/// PUT /api/v1alpha1/tags/{name}
pub async fn update_tag(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(tag): Json<Tag>,
) -> Result<Response, StatusCode> {
    if tag.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match state.tag_service.update(tag).await {
        Ok(tag) => Ok(Json(tag).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除Tag
/// DELETE /api/v1alpha1/tags/{name}
pub async fn delete_tag(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.tag_service.delete(&name).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, "").into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

