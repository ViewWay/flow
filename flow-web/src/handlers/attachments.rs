use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_domain::attachment::{Attachment, ThumbnailSize};
use flow_api::extension::{ListOptions, ListResult};
use flow_api::extension::query::Condition;
use crate::{AppState, extractors::multipart_with_user::MultipartWithUser};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

/// 上传附件
/// POST /api/v1alpha1/attachments
pub async fn upload_attachment(
    State(state): State<AppState>,
    MultipartWithUser { mut multipart, user }: MultipartWithUser,
) -> Result<Response, StatusCode> {
    // 1. 从multipart中提取文件和其他参数
    let mut file_content = Vec::new();
    let mut filename = None;
    let mut media_type = None;
    let mut policy_name = None;
    let mut group_name = None;
    
    while let Some(mut field) = multipart.next_field().await
        .map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name().unwrap_or("");
        
        match field_name {
            "file" | "" => {
                // 提取文件名
                if let Some(name) = field.file_name() {
                    filename = Some(name.to_string());
                }
                
                // 提取Content-Type
                if let Some(content_type) = field.content_type() {
                    media_type = Some(content_type.to_string());
                }
                
                // 读取文件内容
                while let Some(chunk) = field.chunk().await
                    .map_err(|_| StatusCode::BAD_REQUEST)? {
                    file_content.extend_from_slice(&chunk);
                }
            }
            "policyName" => {
                let mut value = String::new();
                while let Some(chunk) = field.chunk().await
                    .map_err(|_| StatusCode::BAD_REQUEST)? {
                    value.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !value.is_empty() {
                    policy_name = Some(value);
                }
            }
            "groupName" => {
                let mut value = String::new();
                while let Some(chunk) = field.chunk().await
                    .map_err(|_| StatusCode::BAD_REQUEST)? {
                    value.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !value.is_empty() {
                    group_name = Some(value);
                }
            }
            _ => {}
        }
    }
    
    let filename = filename.ok_or(StatusCode::BAD_REQUEST)?;
    
    // 2. 验证文件大小（从配置读取，默认100MB）
    const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB
    if file_content.len() > MAX_FILE_SIZE {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    
    // 3. 获取当前用户（如果有）
    let owner_name = user.map(|u| u.username);
    
    // 4. 调用服务上传文件
    match state.attachment_service.upload(
        file_content,
        filename,
        media_type,
        owner_name,
        policy_name,
        group_name,
    ).await {
        Ok(attachment) => Ok(Json(attachment).into_response()),
        Err(e) => {
            eprintln!("Failed to upload attachment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取附件
/// GET /api/v1alpha1/attachments/:name
pub async fn get_attachment(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    match state.attachment_service.get(&name).await {
        Ok(Some(attachment)) => Ok(Json(attachment).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出附件
/// GET /api/v1alpha1/attachments
pub async fn list_attachments(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    // 构建查询条件
    let mut options = ListOptions::default();
    let mut condition = Condition::empty();
    
    // 支持查询参数：groupName, policyName, ownerName, tag等
    if let Some(group_name) = params.get("groupName") {
        condition = condition.and(Condition::Equal {
            index_name: "spec.groupName".to_string(),
            value: serde_json::Value::String(group_name.clone()),
        });
    }
    
    if let Some(policy_name) = params.get("policyName") {
        condition = condition.and(Condition::Equal {
            index_name: "spec.policyName".to_string(),
            value: serde_json::Value::String(policy_name.clone()),
        });
    }
    
    if let Some(owner_name) = params.get("ownerName") {
        condition = condition.and(Condition::Equal {
            index_name: "spec.ownerName".to_string(),
            value: serde_json::Value::String(owner_name.clone()),
        });
    }
    
    if let Some(tag) = params.get("tag") {
        condition = condition.and(Condition::Equal {
            index_name: "spec.tags".to_string(),
            value: serde_json::Value::String(tag.clone()),
        });
    }
    
    if !matches!(condition, Condition::Empty) {
        options.condition = Some(condition);
    }
    
    // 分页参数
    if let Some(page) = params.get("page") {
        options.page = page.parse().ok();
    }
    if let Some(size) = params.get("size") {
        options.size = size.parse().ok();
    }
    
    match state.attachment_service.list(options.clone()).await {
        Ok(attachments) => {
            let total = attachments.len() as u64;
            let page = options.page.unwrap_or(1);
            let size = options.size.unwrap_or(10);
            let result = ListResult::new(attachments, total, page, size);
            Ok(Json(result).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除附件
/// DELETE /api/v1alpha1/attachments/:name
pub async fn delete_attachment(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    match state.attachment_service.delete(&name).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 更新附件
/// PUT /api/v1alpha1/attachments/:name
pub async fn update_attachment(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(attachment): Json<Attachment>,
) -> Result<Response, StatusCode> {
    // 确保name匹配
    if attachment.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match state.attachment_service.update(attachment).await {
        Ok(attachment) => Ok(Json(attachment).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取缩略图
/// GET /api/v1alpha1/attachments/:name/thumbnails/:size
pub async fn get_thumbnail(
    Path((name, size)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    // 1. 获取Attachment
    let attachment = match state.attachment_service.get(&name).await {
        Ok(Some(att)) => att,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 2. 解析缩略图尺寸
    let _thumbnail_size = ThumbnailSize::from_str(&size)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // 3. 从status中获取缩略图URL
    if let Some(ref status) = attachment.status {
        if let Some(ref thumbnails) = status.thumbnails {
            if let Some(thumbnail_url) = thumbnails.get(&size.to_uppercase()) {
                // 返回重定向到缩略图URL
                // 注意：实际实现中，缩略图应该通过静态资源服务提供
                // 这里返回URL，客户端可以重定向访问
                return Ok((
                    StatusCode::TEMPORARY_REDIRECT,
                    [("Location", thumbnail_url.as_str())],
                    thumbnail_url.clone(),
                ).into_response());
            }
        }
    }
    
    // 4. 如果没有缩略图，返回404
    Err(StatusCode::NOT_FOUND)
}

/// 生成共享URL
/// POST /api/v1alpha1/attachments/:name/shared-urls
#[derive(Deserialize)]
pub struct GenerateSharedUrlRequest {
    #[serde(rename = "expiresInHours")]
    pub expires_in_hours: Option<u32>,
}

pub async fn generate_shared_url(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<GenerateSharedUrlRequest>,
) -> Result<Response, StatusCode> {
    // 验证附件是否存在
    match state.attachment_service.get(&name).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    
    // 生成共享URL
    match state.shared_url_service.generate_shared_url(&name, request.expires_in_hours) {
        Ok(shared_url) => Ok(Json(shared_url).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取附件的所有共享URL
/// GET /api/v1alpha1/attachments/:name/shared-urls
pub async fn list_shared_urls(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    match state.shared_url_service.get_shared_urls(&name) {
        Ok(urls) => Ok(Json(json!({"items": urls})).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 撤销共享URL
/// DELETE /api/v1alpha1/attachments/shared-urls/:token
pub async fn revoke_shared_url(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    match state.shared_url_service.revoke_shared_url(&token) {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 通过共享URL访问附件
/// GET /api/v1alpha1/attachments/shared/:token
pub async fn get_attachment_by_shared_url(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    // 验证token
    let attachment_name = match state.shared_url_service.validate_token(&token) {
        Ok(Some(name)) => name,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 获取附件
    match state.attachment_service.get(&attachment_name).await {
        Ok(Some(attachment)) => Ok(Json(attachment).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

