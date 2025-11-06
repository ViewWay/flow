use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_api::extension::ListOptions;
use flow_domain::content::SinglePage;
use flow_service::content::SinglePageService;
use crate::AppState;
use serde::{Deserialize, Serialize};

/// SinglePage列表响应
#[derive(Debug, Serialize)]
pub struct SinglePageListResponse {
    pub items: Vec<SinglePage>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 创建SinglePage
/// POST /api/v1alpha1/singlepages
pub async fn create_single_page(
    State(state): State<AppState>,
    Json(page): Json<SinglePage>,
) -> Result<Response, StatusCode> {
    // TODO: 从认证信息中获取当前用户名
    let username = "admin".to_string();
    let mut page = page;
    page.spec.owner = Some(username);
    
    match state.single_page_service.create(page).await {
        Ok(page) => Ok(Json(page).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取SinglePage
/// GET /api/v1alpha1/singlepages/{name}
pub async fn get_single_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.single_page_service.get(&name).await {
        Ok(Some(page)) => Ok(Json(page).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出SinglePages
/// GET /api/v1alpha1/singlepages
pub async fn list_single_pages(
    State(state): State<AppState>,
    Query(params): Query<ListOptions>,
) -> Result<Response, StatusCode> {
    match state.single_page_service.list(params).await {
        Ok(result) => {
            let response = SinglePageListResponse {
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

/// 更新SinglePage
/// PUT /api/v1alpha1/singlepages/{name}
pub async fn update_single_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(page): Json<SinglePage>,
) -> Result<Response, StatusCode> {
    if page.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match state.single_page_service.update(page).await {
        Ok(page) => Ok(Json(page).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除SinglePage
/// DELETE /api/v1alpha1/singlepages/{name}
pub async fn delete_single_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.single_page_service.delete(&name).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, "").into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 发布SinglePage
/// PUT /api/v1alpha1/singlepages/{name}/publish
pub async fn publish_single_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    let page = match state.single_page_service.get(&name).await {
        Ok(Some(page)) => page,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    match state.single_page_service.publish(page).await {
        Ok(page) => Ok(Json(page).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 取消发布SinglePage
/// PUT /api/v1alpha1/singlepages/{name}/unpublish
pub async fn unpublish_single_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    let page = match state.single_page_service.get(&name).await {
        Ok(Some(page)) => page,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    match state.single_page_service.unpublish(page).await {
        Ok(page) => Ok(Json(page).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

