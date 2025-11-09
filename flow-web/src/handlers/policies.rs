use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use flow_domain::attachment::Policy;
use flow_service::attachment::PolicyService;
use flow_api::extension::ListOptions;
use crate::AppState;
use serde_json::json;
use std::collections::HashMap;

/// 列出Policy
/// GET /api/v1alpha1/policies
pub async fn list_policies(
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
    
    match state.policy_service.list(options.clone()).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to list policies: {}", e)})),
        ).into_response(),
    }
}

/// 获取Policy
/// GET /api/v1alpha1/policies/:name
pub async fn get_policy(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.policy_service.get(&name).await {
        Ok(Some(policy)) => (StatusCode::OK, Json(policy)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Policy not found: {}", name)})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to get policy: {}", e)})),
        ).into_response(),
    }
}

/// 创建Policy
/// POST /api/v1alpha1/policies
pub async fn create_policy(
    State(state): State<AppState>,
    Json(policy): Json<Policy>,
) -> impl IntoResponse {
    match state.policy_service.create(policy).await {
        Ok(policy) => (StatusCode::CREATED, Json(policy)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to create policy: {}", e)})),
        ).into_response(),
    }
}

/// 更新Policy
/// PUT /api/v1alpha1/policies/:name
pub async fn update_policy(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(mut policy): Json<Policy>,
) -> impl IntoResponse {
    // 确保name匹配
    if policy.metadata.name != name {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Policy name mismatch"})),
        ).into_response();
    }
    
    match state.policy_service.update(policy).await {
        Ok(policy) => (StatusCode::OK, Json(policy)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to update policy: {}", e)})),
        ).into_response(),
    }
}

/// 删除Policy
/// DELETE /api/v1alpha1/policies/:name
pub async fn delete_policy(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.policy_service.delete(&name).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to delete policy: {}", e)})),
        ).into_response(),
    }
}

