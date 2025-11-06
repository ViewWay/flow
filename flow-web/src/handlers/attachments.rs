use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_domain::attachment::Attachment;
use flow_service::attachment::AttachmentService;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 上传附件
/// POST /api/v1alpha1/attachments
pub async fn upload_attachment(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response, StatusCode> {
    // TODO: 从AppState获取attachment_service
    // TODO: 实现文件上传逻辑
    // 1. 从multipart中提取文件
    // 2. 验证文件类型和大小
    // 3. 保存文件
    // 4. 生成缩略图（如果是图片）
    // 5. 创建Attachment Extension
    // 6. 返回Attachment对象
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 获取附件
/// GET /api/v1alpha1/attachments/:name
pub async fn get_attachment(
    Path(name): Path<String>,
    State(_state): State<AppState>,
) -> Result<Response, StatusCode> {
    // TODO: 实现获取附件逻辑
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 列出附件
/// GET /api/v1alpha1/attachments
pub async fn list_attachments(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> Result<Response, StatusCode> {
    // TODO: 实现列出附件逻辑
    // 支持查询参数：groupName, policyName, ownerName, tag等
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 删除附件
/// DELETE /api/v1alpha1/attachments/:name
pub async fn delete_attachment(
    Path(name): Path<String>,
    State(_state): State<AppState>,
) -> Result<Response, StatusCode> {
    // TODO: 实现删除附件逻辑
    // 1. 删除文件
    // 2. 删除缩略图
    // 3. 删除Attachment Extension
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 更新附件
/// PUT /api/v1alpha1/attachments/:name
pub async fn update_attachment(
    Path(name): Path<String>,
    State(_state): State<AppState>,
    Json(attachment): Json<Attachment>,
) -> Result<Response, StatusCode> {
    // TODO: 实现更新附件逻辑（主要是更新metadata、tags等）
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 获取缩略图
/// GET /api/v1alpha1/attachments/:name/thumbnails/:size
pub async fn get_thumbnail(
    Path((name, size)): Path<(String, String)>,
    State(_state): State<AppState>,
) -> Result<Response, StatusCode> {
    // TODO: 实现获取缩略图逻辑
    // 1. 获取Attachment
    // 2. 检查缩略图是否存在
    // 3. 如果不存在，生成缩略图
    // 4. 返回缩略图文件
    Err(StatusCode::NOT_IMPLEMENTED)
}

