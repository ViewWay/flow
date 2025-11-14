use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_domain::content::Comment;
use flow_api::extension::ListOptions;
use crate::AppState;
use serde::Serialize;

/// Comment列表响应
#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub items: Vec<Comment>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 创建Comment
/// POST /api/v1alpha1/comments
pub async fn create_comment(
    State(state): State<AppState>,
    Json(comment): Json<Comment>,
) -> Result<Response, StatusCode> {
    match state.comment_service.create(comment).await {
        Ok(comment) => Ok(Json(comment).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取Comment
/// GET /api/v1alpha1/comments/{name}
pub async fn get_comment(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.comment_service.get(&name).await {
        Ok(Some(comment)) => Ok(Json(comment).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出Comments
/// GET /api/v1alpha1/comments
pub async fn list_comments(
    State(state): State<AppState>,
    Query(params): Query<ListOptions>,
) -> Result<Response, StatusCode> {
    match state.comment_service.list(params).await {
        Ok(result) => {
            let response = CommentListResponse {
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

/// 更新Comment
/// PUT /api/v1alpha1/comments/{name}
pub async fn update_comment(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(comment): Json<Comment>,
) -> Result<Response, StatusCode> {
    if comment.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match state.comment_service.update(comment).await {
        Ok(comment) => Ok(Json(comment).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除Comment
/// DELETE /api/v1alpha1/comments/{name}
pub async fn delete_comment(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.comment_service.delete(&name).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, "").into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 批准Comment
/// PUT /api/v1alpha1/comments/{name}/approve
pub async fn approve_comment(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    let comment = match state.comment_service.get(&name).await {
        Ok(Some(comment)) => comment,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    match state.comment_service.approve(comment).await {
        Ok(comment) => Ok(Json(comment).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

