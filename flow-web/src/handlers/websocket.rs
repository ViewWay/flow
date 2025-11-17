use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use axum::http::{StatusCode, HeaderMap};
use flow_api::security::{AuthRequest, AuthenticationResult, RequestInfo};
use flow_infra::websocket::{WebSocketEndpoint, WebSocketMessage, WebSocketSender, WebSocketReceiver};
use futures_util::{SinkExt, StreamExt};
use crate::AppState;
use std::collections::HashMap;
use async_trait::async_trait;

/// Axum WebSocket发送器包装器
struct AxumWebSocketSender {
    sender: futures_util::stream::SplitSink<WebSocket, Message>,
}

#[async_trait]
impl WebSocketSender for AxumWebSocketSender {
    async fn send(&mut self, message: WebSocketMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let axum_message = match message {
            WebSocketMessage::Text(text) => Message::Text(text),
            WebSocketMessage::Binary(data) => Message::Binary(data),
            WebSocketMessage::Close => Message::Close(None),
            WebSocketMessage::Ping(data) => Message::Ping(data),
            WebSocketMessage::Pong(data) => Message::Pong(data),
        };
        self.sender.send(axum_message).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

/// Axum WebSocket接收器包装器
struct AxumWebSocketReceiver {
    receiver: futures_util::stream::SplitStream<WebSocket>,
}

#[async_trait]
impl WebSocketReceiver for AxumWebSocketReceiver {
    async fn recv(&mut self) -> Option<Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>>> {
        use futures_util::StreamExt;
        self.receiver.next().await.map(|result| {
            result.map(|msg| {
                match msg {
                    Message::Text(text) => WebSocketMessage::Text(text),
                    Message::Binary(data) => WebSocketMessage::Binary(data),
                    Message::Close(_) => WebSocketMessage::Close,
                    Message::Ping(data) => WebSocketMessage::Ping(data),
                    Message::Pong(data) => WebSocketMessage::Pong(data),
                }
            }).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })
    }
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
        Ok(AuthenticationResult::RequiresTwoFactor(_)) => {
            // 需要2FA验证，拒绝WebSocket连接
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized: Two-factor authentication required".into())
                .unwrap();
        }
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
    // 分离WebSocket的发送器和接收器
    let (sender, receiver) = socket.split();
    
    // 创建包装器
    let ws_sender: Box<dyn WebSocketSender> = Box::new(AxumWebSocketSender { sender });
    let ws_receiver: Box<dyn WebSocketReceiver> = Box::new(AxumWebSocketReceiver { receiver });
    
    // 调用endpoint的handle_connection方法
    endpoint.handle_connection(ws_sender, ws_receiver).await;
}


