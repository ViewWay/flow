use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_api::extension::{Extension, ExtensionClient, GroupVersionKind, ListOptions};
use crate::{AppState, handlers::extension_utils::DynamicExtension};
use serde_json::Value;
use std::collections::HashMap;

/// Extension端点路径参数
#[derive(Debug)]
pub struct ExtensionPath {
    pub group: String,
    pub version: String,
    pub resource: String,
    pub name: Option<String>,
}

/// 解析Extension路径
/// 格式: /apis/{group}/{version}/{resource} 或 /apis/{group}/{version}/{resource}/{name}
fn parse_extension_path(path: &str) -> Option<ExtensionPath> {
    let parts: Vec<&str> = path.strip_prefix("/apis/")?.split('/').collect();
    if parts.len() < 3 {
        return None;
    }
    
    let group = parts[0].to_string();
    let version = parts[1].to_string();
    let resource = parts[2].to_string();
    let name = parts.get(3).map(|s| s.to_string());
    
    Some(ExtensionPath {
        group,
        version,
        resource,
        name,
    })
}

/// 获取Extension
/// GET /apis/{group}/{version}/{resource}/{name}
pub async fn get_extension(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    let extension_path = parse_extension_path(&path)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let name = extension_path.name
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // 构建扩展对象的完整名称（GVK格式: {group}/{version}/{name}）
    let full_name = format!("{}/{}/{}", extension_path.group, extension_path.version, name);
    
    // 使用DynamicExtension获取扩展对象
    match state.extension_client.fetch::<DynamicExtension>(&full_name).await {
        Ok(Some(extension)) => Ok(Json(extension.to_value()).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出Extensions
/// GET /apis/{group}/{version}/{resource}
pub async fn list_extensions(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    let extension_path = parse_extension_path(&path)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    if extension_path.name.is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 构建ListOptions
    let mut options = ListOptions::default();
    
    // 解析查询参数
    if let Some(label_selector) = params.get("labelSelector") {
        options.label_selector = Some(label_selector.clone());
    }
    
    if let Some(field_selector) = params.get("fieldSelector") {
        options.field_selector = Some(field_selector.clone());
    }
    
    if let Some(page_str) = params.get("page") {
        if let Ok(page) = page_str.parse::<u32>() {
            options.page = Some(page);
        }
    }
    
    if let Some(size_str) = params.get("size") {
        if let Ok(size) = size_str.parse::<u32>() {
            options.size = Some(size);
        }
    }
    
    // 构建GVK用于过滤（只返回匹配的resource类型）
    let gvk = GroupVersionKind::new(
        &extension_path.group,
        &extension_path.version,
        &extension_path.resource,
    );
    
    // 使用DynamicExtension列出扩展对象
    match state.extension_client.list::<DynamicExtension>(options).await {
        Ok(result) => {
            // 过滤结果，只返回匹配GVK的扩展对象
            let filtered_items: Vec<Value> = result.items
                .into_iter()
                .filter(|ext| {
                    let ext_gvk = ext.group_version_kind();
                    ext_gvk.group == gvk.group 
                        && ext_gvk.version == gvk.version 
                        && ext_gvk.kind == gvk.kind
                })
                .map(|ext| ext.to_value())
                .collect();
            
            let response = serde_json::json!({
                "items": filtered_items,
                "total": filtered_items.len() as u64,
                "page": result.page,
                "size": result.size,
            });
            
            Ok(Json(response).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 创建Extension
/// POST /apis/{group}/{version}/{resource}
pub async fn create_extension(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(extension): Json<Value>,
) -> Result<Response, StatusCode> {
    let extension_path = parse_extension_path(&path)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    if extension_path.name.is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 验证extension的GVK是否匹配路径
    let expected_gvk = GroupVersionKind::new(
        &extension_path.group,
        &extension_path.version,
        &extension_path.resource,
    );
    
    // 转换为DynamicExtension
    let mut dynamic_ext = DynamicExtension::from_value(extension)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // 验证GVK匹配
    let actual_gvk = dynamic_ext.group_version_kind();
    if actual_gvk != expected_gvk {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 创建扩展对象
    match state.extension_client.create(dynamic_ext).await {
        Ok(extension) => Ok(Json(extension.to_value()).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 更新Extension
/// PUT /apis/{group}/{version}/{resource}/{name}
pub async fn update_extension(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(extension): Json<Value>,
) -> Result<Response, StatusCode> {
    let extension_path = parse_extension_path(&path)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let name = extension_path.name
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // 验证extension的GVK是否匹配路径
    let expected_gvk = GroupVersionKind::new(
        &extension_path.group,
        &extension_path.version,
        &extension_path.resource,
    );
    
    // 转换为DynamicExtension
    let mut dynamic_ext = DynamicExtension::from_value(extension)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // 验证GVK和name匹配
    let actual_gvk = dynamic_ext.group_version_kind();
    if actual_gvk != expected_gvk {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if dynamic_ext.metadata.name != name {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 更新扩展对象
    match state.extension_client.update(dynamic_ext).await {
        Ok(extension) => Ok(Json(extension.to_value()).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除Extension
/// DELETE /apis/{group}/{version}/{resource}/{name}
pub async fn delete_extension(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    let extension_path = parse_extension_path(&path)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let name = extension_path.name
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // 构建扩展对象的完整名称（GVK格式: {group}/{version}/{name}）
    let full_name = format!("{}/{}/{}", extension_path.group, extension_path.version, name);
    
    // 删除扩展对象（使用DynamicExtension作为类型参数）
    match state.extension_client.delete::<DynamicExtension>(&full_name).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 补丁Extension（PATCH）
/// PATCH /apis/{group}/{version}/{resource}/{name}
pub async fn patch_extension(
    State(_state): State<AppState>,
    Path(_path): Path<String>,
    Json(_patch): Json<Value>,
) -> Result<Response, StatusCode> {
    // TODO: 实现PATCH操作（JSON Patch或JSON Merge Patch）
    // 需要实现JSON Patch或JSON Merge Patch算法
    Err(StatusCode::NOT_IMPLEMENTED)
}

