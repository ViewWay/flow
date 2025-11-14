use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use flow_domain::attachment::Group;
use flow_api::extension::ListOptions;
use crate::AppState;
use serde_json::json;
use std::collections::HashMap;

/// 列出Group
/// GET /api/v1alpha1/groups
pub async fn list_groups(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut options = ListOptions::default();
    
    // 分页参数
    if let Some(page) = params.get("page") {
        options.page = page.parse().ok();
    }
    if let Some(size) = params.get("size") {
        options.size = size.parse().ok();
    }
    
    match state.group_service.list(options.clone()).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to list groups: {}", e)})),
        ).into_response(),
    }
}

/// 获取Group
/// GET /api/v1alpha1/groups/:name
pub async fn get_group(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.group_service.get(&name).await {
        Ok(Some(group)) => (StatusCode::OK, Json(group)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Group not found: {}", name)})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to get group: {}", e)})),
        ).into_response(),
    }
}

/// 创建Group
/// POST /api/v1alpha1/groups
pub async fn create_group(
    State(state): State<AppState>,
    Json(group): Json<Group>,
) -> impl IntoResponse {
    match state.group_service.create(group).await {
        Ok(group) => (StatusCode::CREATED, Json(group)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to create group: {}", e)})),
        ).into_response(),
    }
}

/// 更新Group
/// PUT /api/v1alpha1/groups/:name
pub async fn update_group(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(group): Json<Group>,
) -> impl IntoResponse {
    // 确保name匹配
    if group.metadata.name != name {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Group name mismatch"})),
        ).into_response();
    }
    
    match state.group_service.update(group).await {
        Ok(group) => (StatusCode::OK, Json(group)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to update group: {}", e)})),
        ).into_response(),
    }
}

/// 删除Group
/// DELETE /api/v1alpha1/groups/:name
pub async fn delete_group(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.group_service.delete(&name).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to delete group: {}", e)})),
        ).into_response(),
    }
}

/// 更新Group的附件计数
/// POST /api/v1alpha1/groups/:name/update-count
pub async fn update_group_count(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.group_service.update_attachment_count(&name).await {
        Ok(group) => (StatusCode::OK, Json(group)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to update group count: {}", e)})),
        ).into_response(),
    }
}

