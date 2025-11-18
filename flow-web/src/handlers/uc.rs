use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_domain::content::{Post, Snapshot};
use flow_service::content::{PostQuery, PostRequest, ContentRequest};
use crate::{AppState, extractors::CurrentUser};
use serde::{Deserialize, Serialize};

/// 创建我的Post（草稿）
/// POST /api/v1alpha1/uc/posts
pub async fn create_my_post(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Json(post): Json<Post>,
) -> Result<Response, StatusCode> {
    
    // 设置Post的owner为当前用户
    let mut post = post;
    if post.spec.owner.is_none() {
        post.spec.owner = Some(username.clone());
    } else if post.spec.owner.as_ref() != Some(&username) {
        // 不允许设置owner为其他用户
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 检查是否有内容（从annotations中提取）
    let content = extract_content_from_post(&mut post);
    
    let post_request = PostRequest {
        post,
        content: content.map(|c| ContentRequest {
            raw: c.raw,
            content: c.content,
            raw_type: c.raw_type,
        }),
    };
    
    match state.post_service.draft_post(post_request).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取我的Post
/// GET /api/v1alpha1/uc/posts/{name}
pub async fn get_my_post(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    
    match state.post_service.get_by_username(&name, &username).await {
        Ok(Some(post)) => Ok(Json(post).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出我的Posts
/// GET /api/v1alpha1/uc/posts
pub async fn list_my_posts(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Query(params): Query<serde_json::Value>,
) -> Result<Response, StatusCode> {
    
    // 解析查询参数
    let mut query = parse_post_query(params);
    // UC端点只返回当前用户的posts
    query.owner = Some(username);
    
    match state.post_service.list_post(query).await {
        Ok(result) => {
            let response = UcPostListResponse {
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

/// 更新我的Post
/// PUT /api/v1alpha1/uc/posts/{name}
pub async fn update_my_post(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Path(name): Path<String>,
    Json(mut post): Json<Post>,
) -> Result<Response, StatusCode> {
    
    // 确保name匹配
    if post.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 获取原始Post以限制可更新字段
    let old_post = match state.post_service.get_by_username(&name, &username).await {
        Ok(Some(p)) => p,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 限制字段更新（不允许修改owner、publish状态等）
    let old_spec = &old_post.spec;
    post.spec.owner = old_spec.owner.clone();
    post.spec.publish = old_spec.publish;
    post.spec.head_snapshot = old_spec.head_snapshot.clone();
    post.spec.base_snapshot = old_spec.base_snapshot.clone();
    post.spec.release_snapshot = old_spec.release_snapshot.clone();
    post.spec.deleted = old_spec.deleted;
    
    // 移除content annotation（UC端点不支持在更新Post时更新内容）
    if let Some(annotations) = &mut post.metadata.annotations {
        annotations.remove("content.halo.run/content-json");
    }
    
    let post_request = PostRequest {
        post,
        content: None, // UC端点更新Post时不更新内容
    };
    
    match state.post_service.update_post(post_request).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 发布我的Post
/// PUT /api/v1alpha1/uc/posts/{name}/publish
pub async fn publish_my_post(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    
    let post = match state.post_service.get_by_username(&name, &username).await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    match state.post_service.publish(post).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 取消发布我的Post
/// PUT /api/v1alpha1/uc/posts/{name}/unpublish
pub async fn unpublish_my_post(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    
    let post = match state.post_service.get_by_username(&name, &username).await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    match state.post_service.unpublish(post).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 回收我的Post（移到回收站）
/// DELETE /api/v1alpha1/uc/posts/{name}/recycle
pub async fn recycle_my_post(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    
    match state.post_service.recycle(&name, &username).await {
        Ok(post) => Ok(Json(post).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取我的Post草稿
/// GET /api/v1alpha1/uc/posts/{name}/draft
pub async fn get_my_post_draft(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Path(name): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    
    // 获取Post
    let post = match state.post_service.get_by_username(&name, &username).await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 获取head snapshot或base snapshot
    let snapshot_name = post.spec.head_snapshot
        .as_ref()
        .or(post.spec.base_snapshot.as_ref())
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let patched = params.get("patched")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);
    
    if patched {
        // 获取合并后的内容
        let base_snapshot = post.spec.base_snapshot.as_deref();
        match state.post_service.get_content(snapshot_name, base_snapshot).await {
            Ok(content) => Ok(Json(content).into_response()),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        // 获取原始snapshot
        match state.snapshot_service.get(snapshot_name).await {
            Ok(Some(snapshot)) => Ok(Json(snapshot).into_response()),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

/// 更新我的Post草稿
/// PUT /api/v1alpha1/uc/posts/{name}/draft
pub async fn update_my_post_draft(
    State(state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Path(name): Path<String>,
    Json(snapshot): Json<Snapshot>,
) -> Result<Response, StatusCode> {
    
    // 获取Post
    let post = match state.post_service.get_by_username(&name, &username).await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 验证snapshot属于该Post
    if snapshot.spec.subject_ref.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 验证是head snapshot
    let head_snapshot_name = post.spec.head_snapshot.as_ref()
        .ok_or(StatusCode::BAD_REQUEST)?;
    if head_snapshot_name != &snapshot.metadata.name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 获取base snapshot
    let base_snapshot_name = post.spec.base_snapshot.as_ref()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let base_snapshot = match state.snapshot_service.get(base_snapshot_name).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 获取head snapshot
    let head_snapshot = match state.snapshot_service.get(head_snapshot_name).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 从请求的snapshot中提取内容
    let new_raw = snapshot.spec.raw_patch.as_deref().unwrap_or("");
    let new_content = snapshot.spec.content_patch.as_deref().unwrap_or("");
    
    // 获取base snapshot的完整内容（用于计算diff）
    let base_raw = base_snapshot.spec.raw_patch.as_deref().unwrap_or("");
    let base_content = base_snapshot.spec.content_patch.as_deref().unwrap_or("");
    
    // 如果head snapshot不是base snapshot，需要先获取head snapshot的完整内容
    let head_raw = if head_snapshot_name == base_snapshot_name {
        base_raw.to_string()
    } else {
        // 应用head snapshot的patch到base snapshot
        let head_raw_patch = head_snapshot.spec.raw_patch.as_deref().unwrap_or("");
        if head_raw_patch.is_empty() {
            base_raw.to_string()
        } else {
            match flow_service::content::patch_utils::apply_patch(base_raw, head_raw_patch) {
                Ok(r) => r,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    };
    
    let head_content = if head_snapshot_name == base_snapshot_name {
        base_content.to_string()
    } else {
        let head_content_patch = head_snapshot.spec.content_patch.as_deref().unwrap_or("");
        if head_content_patch.is_empty() {
            base_content.to_string()
        } else {
            match flow_service::content::patch_utils::apply_patch(base_content, head_content_patch) {
                Ok(c) => c,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    };
    
    // 计算diff（从head snapshot到新内容）
    let raw_patch = if head_raw == new_raw {
        "[]".to_string()
    } else {
        match flow_service::content::patch_utils::diff_to_json_patch(&head_raw, new_raw) {
            Ok(p) => p,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    };
    
    let content_patch = if head_content == new_content {
        "[]".to_string()
    } else {
        match flow_service::content::patch_utils::diff_to_json_patch(&head_content, new_content) {
            Ok(p) => p,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    };
    
    // 检查head snapshot是否等于release snapshot
    let release_snapshot_name = post.spec.release_snapshot.as_ref();
    let should_create_new = release_snapshot_name.map(|r| r == head_snapshot_name).unwrap_or(false);
    
    if should_create_new {
        // 创建新的snapshot
        use flow_api::extension::Metadata;
        use uuid::Uuid;
        use chrono::Utc;
        
        let mut new_snapshot = Snapshot {
            metadata: Metadata {
                name: format!("{}-snapshot-{}", name, Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
                version: Some(0),
                creation_timestamp: Some(Utc::now()),
                labels: None,
                annotations: None,
            },
            spec: snapshot.spec.clone(),
        };
        
        new_snapshot.spec.parent_snapshot_name = Some(head_snapshot_name.clone());
        new_snapshot.spec.raw_patch = Some(raw_patch);
        new_snapshot.spec.content_patch = Some(content_patch);
        new_snapshot.spec.last_modify_time = Some(Utc::now());
        new_snapshot.spec.owner = username.clone();
        new_snapshot.add_contributor(username.clone());
        
        let created_snapshot = match state.snapshot_service.create(new_snapshot).await {
            Ok(s) => s,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        
        // 更新Post的head_snapshot
        let mut updated_post = post;
        updated_post.spec.head_snapshot = Some(created_snapshot.metadata.name.clone());
        match state.post_service.update_by(updated_post).await {
            Ok(_) => Ok(Json(created_snapshot).into_response()),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        // 更新现有的snapshot
        let mut updated_snapshot = head_snapshot;
        updated_snapshot.spec.raw_patch = Some(raw_patch);
        updated_snapshot.spec.content_patch = Some(content_patch);
        updated_snapshot.spec.last_modify_time = Some(chrono::Utc::now());
        updated_snapshot.add_contributor(username.clone());
        
        match state.snapshot_service.update(updated_snapshot).await {
            Ok(s) => Ok(Json(s).into_response()),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

/// Post列表响应
#[derive(Debug, Serialize)]
pub struct UcPostListResponse {
    pub items: Vec<flow_service::content::ListedPost>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 内容结构（从Post annotations中提取）
#[derive(Debug, Deserialize)]
struct Content {
    raw: String,
    content: String,
    #[serde(rename = "rawType")]
    raw_type: String,
}

/// 从Post的annotations中提取内容
fn extract_content_from_post(post: &mut Post) -> Option<Content> {
    post.metadata.annotations
        .as_mut()?
        .remove("content.halo.run/content-json")
        .and_then(|json| serde_json::from_str(&json).ok())
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

