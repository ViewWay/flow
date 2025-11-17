use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use flow_api::security::{AuthRequest, AuthenticationResult};
use crate::AppState;
use std::collections::HashMap;

/// 认证中间件
/// 从请求中提取认证信息，调用认证服务，将用户信息注入请求扩展
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // 提取请求信息
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    
    // 提取所有头信息
    let mut headers = HashMap::new();
    for (name, value) in request.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // 构建AuthRequest
    let auth_request = AuthRequest {
        method,
        path,
        headers,
        body: None, // 中间件中不处理body
    };

    // 调用认证服务
    match state.auth_service.authenticate(&auth_request).await {
        Ok(AuthenticationResult::Authenticated(user)) => {
            // 认证成功，将用户信息注入请求扩展
            request.extensions_mut().insert(user);
        }
        Ok(AuthenticationResult::RequiresTwoFactor(_)) => {
            // 需要2FA验证，但允许继续（可能是公开端点）
            // 授权中间件会处理权限检查
        }
        Ok(AuthenticationResult::Unauthenticated) => {
            // 未认证，但允许继续（可能是公开端点）
            // 授权中间件会处理权限检查
        }
        Ok(AuthenticationResult::Failed(_)) => {
            // 认证失败，但允许继续（可能是公开端点）
            // 授权中间件会处理权限检查
        }
        Err(_) => {
            // 认证服务错误，但允许继续
        }
    }

    // 继续处理请求
    next.run(request).await
}

