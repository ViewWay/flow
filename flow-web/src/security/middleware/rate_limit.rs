use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::http::StatusCode;
use crate::AppState;

/// 速率限制中间件
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // 获取客户端IP（简化处理，实际应该从X-Forwarded-For等头中提取）
    let client_ip = request.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            request.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
        })
        .unwrap_or("unknown");

    // 调用速率限制器
    match state.rate_limiter.check(
        client_ip,
        100, // limit
        60,  // window_seconds
    ).await {
        Ok((allowed, _remaining, reset_time)) => {
            if !allowed {
                return Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("X-RateLimit-Limit", "100")
                    .header("X-RateLimit-Remaining", "0")
                    .header("X-RateLimit-Reset", &reset_time.to_string())
                    .body(axum::body::Body::from("Too many requests"))
                    .unwrap();
            }
        }
        Err(_) => {
            // 速率限制检查错误，允许继续（避免因基础设施问题影响服务）
        }
    }

    // 继续处理请求
    next.run(request).await
}

