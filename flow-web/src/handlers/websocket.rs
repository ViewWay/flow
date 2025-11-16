use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use axum::http::{StatusCode, HeaderMap};
use flow_api::security::{AuthRequest, AuthenticationResult, RequestInfo};
use flow_infra::websocket::WebSocketEndpoint;
use futures_util::{SinkExt, StreamExt};
use crate::AppState;
use std::collections::HashMap;

/// WebSocket端点扩展trait
/// 为WebSocketEndpoint添加handler方法，在flow-web层实现
pub trait WebSocketEndpointExt: WebSocketEndpoint {
    /// 处理WebSocket连接
    /// 这个方法会被调用以处理WebSocket消息
    async fn handle_connection(&self, socket: WebSocket);
}

/// WebSocket连接处理器
/// 处理 /apis/{group}/{version}/{path} 的WebSocket连接
/// 在升级WebSocket之前进行认证和权限检查
pub async fn handle_websocket(
    Path(path): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    // 查找端点
    let endpoint = match state.websocket_manager.find(&path).await {
        Some(ep) => ep,
        None => {
            // 如果没有找到WebSocket端点，返回404
            // 注意：这会让请求fallback到Extension路由（如果存在）
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("WebSocket endpoint not found".into())
                .unwrap();
        }
    };
    
    // 1. 认证检查
    // 提取所有头信息
    let mut header_map = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(name.to_string(), value_str.to_string());
        }
    }
    
    // 构建AuthRequest（WebSocket使用GET方法）
    let auth_request = AuthRequest {
        method: "GET".to_string(),
        path: format!("/apis/{}", path),
        headers: header_map,
        body: None,
    };
    
    // 调用认证服务
    let user = match state.auth_service.authenticate(&auth_request).await {
        Ok(AuthenticationResult::Authenticated(user)) => user,
        Ok(AuthenticationResult::Unauthenticated) => {
            // 未认证，拒绝WebSocket连接
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized: WebSocket requires authentication".into())
                .unwrap();
        }
        Ok(AuthenticationResult::Failed(reason)) => {
            // 认证失败，拒绝WebSocket连接
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(format!("Authentication failed: {}", reason).into())
                .unwrap();
        }
        Err(e) => {
            // 认证服务错误
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Authentication error: {}", e).into())
                .unwrap();
        }
    };
    
    // 2. 权限检查
    // 构建RequestInfo（WebSocket使用GET方法）
    let request_info = RequestInfo::from_request("GET", &format!("/apis/{}", path));
    
    // 调用授权管理器
    match state.authorization_manager.check(&user, &request_info).await {
        Ok(decision) => {
            if !decision.allowed {
                // 权限检查失败，拒绝WebSocket连接
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(
                        decision.reason
                            .unwrap_or_else(|| "Forbidden: insufficient permissions".to_string())
                            .into()
                    )
                    .unwrap();
            }
        }
        Err(e) => {
            // 授权检查错误
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Authorization error: {}", e).into())
                .unwrap();
        }
    }
    
    // 3. 认证和权限检查通过，升级WebSocket连接
    ws.on_upgrade(move |socket| async move {
        handle_websocket_connection(socket, endpoint).await;
    })
}

/// 处理WebSocket连接
async fn handle_websocket_connection(
    socket: WebSocket, 
    endpoint: std::sync::Arc<dyn flow_infra::websocket::WebSocketEndpoint>
) {
    // 尝试通过Any trait进行downcast来调用自定义handler
    // 如果endpoint实现了WebSocketEndpointExt，调用其handler
    // 否则使用默认的echo处理
    
    // 使用as_any获取Any引用，然后尝试downcast到具体的类型
    // 由于我们需要调用async方法，我们需要知道具体的类型
    // 这里我们尝试downcast到已知的类型（如EchoEndpoint）
    
    // 尝试调用自定义handler
    // 注意：由于Rust的类型系统限制，我们需要知道具体的类型才能调用async方法
    // 这里我们尝试downcast到已知的类型
    
    // 由于Arc<dyn Trait>的限制，我们需要使用类型擦除和downcast
    // 但downcast需要知道具体类型，所以我们尝试downcast到已知的类型
    // 如果无法匹配，则使用默认处理
    
    // 尝试downcast到EchoEndpoint（在server.rs中定义）
    // 注意：这需要EchoEndpoint在flow-web层可见，或者我们需要一个更好的机制
    // 暂时使用默认处理，实际的endpoint会在server.rs中实现WebSocketEndpointExt trait
    default_websocket_handler(socket).await;
}

/// 默认WebSocket处理器（echo消息）
/// 如果endpoint没有实现自定义handler，使用这个默认处理器
async fn default_websocket_handler(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Echo消息
                if sender.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // 忽略Pong消息
            }
            Ok(Message::Binary(_)) => {
                // 忽略二进制消息（或根据endpoint处理）
            }
            Err(_) => {
                break;
            }
        }
    }
}

