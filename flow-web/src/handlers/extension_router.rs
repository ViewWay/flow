use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::AppState;
use serde_json::Value;
use std::collections::HashMap;

/// 处理Extension GET请求
/// 根据路径长度路由到get_extension或list_extensions
/// Path提取器会提取通配符部分，例如：/apis/content.halo.run/v1alpha1/posts/my-post
/// 提取的path是: content.halo.run/v1alpha1/posts/my-post
pub async fn handle_extension_get(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    use crate::handlers::extension::{get_extension, list_extensions};
    
    // 路径格式: {group}/{version}/{resource} 或 {group}/{version}/{resource}/{name}
    let parts: Vec<&str> = path.split('/').collect();
    
    // 构建完整路径（加上/apis前缀）
    let full_path = format!("/apis/{}", path);
    
    if parts.len() == 4 {
        // {group}/{version}/{resource}/{name}
        get_extension(State(state), Path(full_path)).await
    } else if parts.len() == 3 {
        // {group}/{version}/{resource}
        list_extensions(State(state), Path(full_path), Query(params)).await
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

/// 处理Extension POST请求
pub async fn handle_extension_post(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(extension): Json<Value>,
) -> Result<Response, StatusCode> {
    use crate::handlers::extension::create_extension;
    
    // 构建完整路径（加上/apis前缀）
    let full_path = format!("/apis/{}", path);
    create_extension(State(state), Path(full_path), Json(extension)).await
}

/// 处理Extension PUT请求
pub async fn handle_extension_put(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(extension): Json<Value>,
) -> Result<Response, StatusCode> {
    use crate::handlers::extension::update_extension;
    
    // 构建完整路径（加上/apis前缀）
    let full_path = format!("/apis/{}", path);
    update_extension(State(state), Path(full_path), Json(extension)).await
}

/// 处理Extension DELETE请求
pub async fn handle_extension_delete(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    use crate::handlers::extension::delete_extension;
    
    // 构建完整路径（加上/apis前缀）
    let full_path = format!("/apis/{}", path);
    delete_extension(State(state), Path(full_path)).await
}

/// 处理Extension PATCH请求
pub async fn handle_extension_patch(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(patch): Json<Value>,
) -> Result<Response, StatusCode> {
    use crate::handlers::extension::patch_extension;
    
    // 构建完整路径（加上/apis前缀）
    let full_path = format!("/apis/{}", path);
    patch_extension(State(state), Path(full_path), Json(patch)).await
}

