use axum::{
    extract::{Path, Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use crate::AppState;
use serde_json::Value;
use std::collections::HashMap;

/// 处理Extension GET请求或WebSocket升级
/// 根据请求头判断是WebSocket升级还是普通HTTP请求
pub async fn handle_extension_get(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    request: axum::extract::Request,
) -> Result<Response, StatusCode> {
    // 检查是否是WebSocket升级请求
    if is_websocket_upgrade(&headers) {
        // 处理WebSocket升级
        // 尝试提取WebSocketUpgrade
        // 注意：这里需要从请求中提取WebSocketUpgrade
        // 由于Axum的限制，我们需要在路由层面处理WebSocket
        // 这里返回错误，让调用者知道需要特殊处理
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 特殊处理：适配Halo前端的Extension API格式
    // /apis/api.console.halo.run/v1alpha1/users/- -> 获取当前用户详情
    if path == "api.console.halo.run/v1alpha1/users/-" {
        return handle_get_current_user_detail(State(state), request).await;
    }
    
    // 普通HTTP GET请求
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

/// 处理获取当前用户详情（Extension API格式）
/// GET /apis/api.console.halo.run/v1alpha1/users/-
async fn handle_get_current_user_detail(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Response, StatusCode> {
    use flow_api::security::AuthenticatedUser;
    use flow_domain::security::User;
    use serde_json::json;
    
    // 从请求扩展中获取已认证的用户
    let authenticated_user = match request.extensions().get::<AuthenticatedUser>() {
        Some(user) => user,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // 获取用户详细信息
    let user = match state.user_service.get(&authenticated_user.username).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 获取用户角色
    let roles = match state.role_service.get_user_roles(&authenticated_user.username).await {
        Ok(roles) => {
            // 将角色名称转换为Role对象（简化版，只包含name）
            roles.into_iter().map(|role_name| {
                json!({
                    "metadata": {
                        "name": role_name
                    }
                })
            }).collect::<Vec<_>>()
        },
        Err(_) => vec![],
    };
    
    // 转换为DetailedUser格式（Halo前端期望的格式）
    // DetailedUser { user: User, roles: Array<Role> }
    let detailed_user = json!({
        "user": user,
        "roles": roles
    });
    
    Ok(Json(detailed_user).into_response())
}

/// 检查是否是WebSocket升级请求
fn is_websocket_upgrade(headers: &HeaderMap) -> bool {
    headers.get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
    && headers.get("connection")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("Upgrade") || v.contains("upgrade"))
        .unwrap_or(false)
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

