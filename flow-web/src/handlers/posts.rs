use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_api::extension::ListOptions;
use flow_domain::content::Post;
use flow_service::content::{PostService, PostQuery, PostRequest, ContentWrapper, ContentRequest};
use crate::AppState;
use serde::{Deserialize, Serialize};

/// 创建Post请求
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub post: Post,
    pub content: Option<ContentUpdateParam>,
}

/// 内容更新参数
#[derive(Debug, Deserialize)]
pub struct ContentUpdateParam {
    pub raw: String,
    pub content: String,
    #[serde(rename = "rawType")]
    pub raw_type: String,
    pub version: Option<u64>,
}

impl ContentUpdateParam {
    pub fn to_content_request(&self, post: &Post) -> ContentRequest {
        ContentRequest {
            raw: self.raw.clone(),
            content: self.content.clone(),
            raw_type: self.raw_type.clone(),
        }
    }
}

/// 更新Post请求
#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub post: Post,
    pub content: Option<ContentUpdateParam>,
}

/// 发布Post请求
#[derive(Debug, Deserialize)]
pub struct PublishPostRequest {
    #[serde(rename = "headSnapshot")]
    pub head_snapshot: Option<String>,
}

/// 恢复到快照请求
#[derive(Debug, Deserialize)]
pub struct RevertSnapshotRequest {
    #[serde(rename = "snapshotName")]
    pub snapshot_name: String,
}

/// Post列表响应
#[derive(Debug, Serialize)]
pub struct PostListResponse {
    pub items: Vec<flow_service::content::ListedPost>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 创建Post（草稿）
/// POST /api/v1alpha1/posts
pub async fn create_post(
    State(state): State<AppState>,
    Json(request): Json<CreatePostRequest>,
) -> Result<Response, StatusCode> {
    // TODO: 从认证信息中获取当前用户名
    let username = "admin".to_string(); // 临时硬编码
    
    // 设置Post的owner
    let mut post = request.post;
    post.spec.owner = Some(username);
    
    // 构建PostRequest
    let post_request = PostRequest {
        post,
        content: request.content.map(|c| {
            ContentRequest {
                raw: c.raw,
                content: c.content,
                raw_type: c.raw_type,
            }
        }),
    };
    
    match state.post_service.draft_post(post_request).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取Post
/// GET /api/v1alpha1/posts/{name}
pub async fn get_post(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    // TODO: 实现get_by_username以检查权限
    match state.post_service.get_by_username(&name, "admin").await {
        Ok(Some(post)) => Ok(Json(post).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出Posts
/// GET /api/v1alpha1/posts
pub async fn list_posts(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Response, StatusCode> {
    // 解析查询参数
    let query = parse_post_query(params);
    
    match state.post_service.list_post(query).await {
        Ok(result) => {
            let response = PostListResponse {
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

/// 更新Post
/// PUT /api/v1alpha1/posts/{name}
pub async fn update_post(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<UpdatePostRequest>,
) -> Result<Response, StatusCode> {
    // 确保name匹配
    if request.post.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let post_request = PostRequest {
        post: request.post,
        content: request.content.map(|c| {
            ContentRequest {
                raw: c.raw,
                content: c.content,
                raw_type: c.raw_type,
            }
        }),
    };
    
    match state.post_service.update_post(post_request).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 发布Post
/// PUT /api/v1alpha1/posts/{name}/publish
pub async fn publish_post(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    // 获取Post
    let post = match state.post_service.get_by_username(&name, "admin").await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // TODO: 处理headSnapshot参数
    // let head_snapshot = params.get("headSnapshot");
    
    match state.post_service.publish(post).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 取消发布Post
/// PUT /api/v1alpha1/posts/{name}/unpublish
pub async fn unpublish_post(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    let post = match state.post_service.get_by_username(&name, "admin").await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    match state.post_service.unpublish(post).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 回收Post（移到回收站）
/// PUT /api/v1alpha1/posts/{name}/recycle
pub async fn recycle_post(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    // TODO: 从认证信息中获取当前用户名
    let username = "admin".to_string();
    
    match state.post_service.recycle(&name, &username).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除Post
/// DELETE /api/v1alpha1/posts/{name}
pub async fn delete_post(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    // TODO: 实现Post删除（需要先检查是否有依赖）
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 获取Post头部内容
/// GET /api/v1alpha1/posts/{name}/head-content
pub async fn get_post_head_content(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.post_service.get_head_content(&name).await {
        Ok(content) => Ok(Json(content).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取Post发布内容
/// GET /api/v1alpha1/posts/{name}/release-content
pub async fn get_post_release_content(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.post_service.get_release_content(&name).await {
        Ok(content) => Ok(Json(content).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取Post指定快照内容
/// GET /api/v1alpha1/posts/{name}/content?snapshotName=xxx
pub async fn get_post_content(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    let snapshot_name = params.get("snapshotName")
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;
    
    // 获取Post以获取baseSnapshot
    let post = match state.post_service.get_by_username(&name, "admin").await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    let base_snapshot = post.spec.base_snapshot.as_deref();
    
    match state.post_service.get_content(snapshot_name, base_snapshot).await {
        Ok(content) => Ok(Json(content).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 恢复到指定快照
/// PUT /api/v1alpha1/posts/{name}/revert-content
pub async fn revert_post_to_snapshot(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RevertSnapshotRequest>,
) -> Result<Response, StatusCode> {
    match state.post_service.revert_to_snapshot(&name, &request.snapshot_name).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除Post内容（删除指定快照）
/// DELETE /api/v1alpha1/posts/{name}/content?snapshotName=xxx
pub async fn delete_post_content(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    let snapshot_name = params.get("snapshotName")
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;
    
    match state.post_service.delete_content(&name, snapshot_name).await {
        Ok(content) => Ok(Json(content).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 解析Post查询参数
fn parse_post_query(params: serde_json::Value) -> PostQuery {
    let mut query = PostQuery::default();
    
    if let Some(published) = params.get("publishPhase").and_then(|v| v.as_str()) {
        query.published = Some(published == "PUBLISHED");
    }
    
    if let Some(keyword) = params.get("keyword").and_then(|v| v.as_str()) {
        query.keyword = Some(keyword.to_string());
    }
    
    if let Some(category) = params.get("category").and_then(|v| v.as_str()) {
        query.category = Some(category.to_string());
    }
    
    if let Some(tag) = params.get("tag").and_then(|v| v.as_str()) {
        query.tag = Some(tag.to_string());
    }
    
    if let Some(visible) = params.get("visible").and_then(|v| v.as_str()) {
        query.visible = match visible {
            "PUBLIC" => Some(flow_domain::content::VisibleEnum::Public),
            "INTERNAL" => Some(flow_domain::content::VisibleEnum::Internal),
            "PRIVATE" => Some(flow_domain::content::VisibleEnum::Private),
            _ => None,
        };
    }
    
    if let Some(page) = params.get("page").and_then(|v| v.as_u64()) {
        query.page = Some(page as u32);
    }
    
    if let Some(size) = params.get("size").and_then(|v| v.as_u64()) {
        query.size = Some(size as u32);
    }
    
    query
}

