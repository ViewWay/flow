use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_api::extension::ListOptions;
use flow_domain::content::Category;
use flow_service::content::CategoryService;
use crate::AppState;
use serde::{Deserialize, Serialize};

/// Category列表响应
#[derive(Debug, Serialize)]
pub struct CategoryListResponse {
    pub items: Vec<Category>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 创建Category
/// POST /api/v1alpha1/categories
pub async fn create_category(
    State(state): State<AppState>,
    Json(category): Json<Category>,
) -> Result<Response, StatusCode> {
    match state.category_service.create(category).await {
        Ok(category) => Ok(Json(category).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取Category
/// GET /api/v1alpha1/categories/{name}
pub async fn get_category(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.category_service.get(&name).await {
        Ok(Some(category)) => Ok(Json(category).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出Categories
/// GET /api/v1alpha1/categories
pub async fn list_categories(
    State(state): State<AppState>,
    Query(params): Query<ListOptions>,
) -> Result<Response, StatusCode> {
    match state.category_service.list(params).await {
        Ok(result) => {
            let response = CategoryListResponse {
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

/// 更新Category
/// PUT /api/v1alpha1/categories/{name}
pub async fn update_category(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(category): Json<Category>,
) -> Result<Response, StatusCode> {
    if category.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match state.category_service.update(category).await {
        Ok(category) => Ok(Json(category).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除Category
/// DELETE /api/v1alpha1/categories/{name}
pub async fn delete_category(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.category_service.delete(&name).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, "").into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

