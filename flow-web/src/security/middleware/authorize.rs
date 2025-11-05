use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::http::StatusCode;
use flow_api::security::RequestInfo;
use crate::AppState;

/// 授权中间件
/// 检查用户是否有权限访问请求的资源
pub async fn authorize_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // 从请求扩展中获取用户信息
    let user = match request.extensions().get::<flow_api::security::AuthenticatedUser>() {
        Some(user) => user,
        None => {
            // 未认证用户，对于某些公开端点允许访问
            // 这里简化处理，实际应该检查路径是否为公开端点
            let path = request.uri().path();
            if path == "/health" || path == "/api/v1alpha1/health" {
                return next.run(request).await;
            }
            // 其他端点需要认证
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from("Unauthorized"))
                .unwrap();
        }
    };

    // 解析RequestInfo
    let method = request.method().as_str();
    let path = request.uri().path();
    let request_info = RequestInfo::from_request(method, path);

    // 调用授权管理器
    match state.authorization_manager.check(user, &request_info).await {
        Ok(decision) => {
            if !decision.allowed {
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(axum::body::Body::from(
                        decision.reason.unwrap_or_else(|| "Forbidden".to_string())
                    ))
                    .unwrap();
            }
        }
        Err(e) => {
            // 授权检查错误
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from(format!("Authorization error: {}", e)))
                .unwrap();
        }
    }

    // 授权通过，继续处理请求
    next.run(request).await
}

