use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use axum::http::{StatusCode, HeaderMap};
use flow_api::security::{AuthRequest, AuthenticationResult, RequestInfo};
use flow_infra::websocket::WebSocketEndpointManager;
use futures_util::{SinkExt, StreamExt};
use crate::AppState;
use std::collections::HashMap;

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
async fn handle_websocket_connection(socket: WebSocket, _endpoint: std::sync::Arc<dyn flow_infra::websocket::WebSocketEndpoint>) {
    let (mut sender, mut receiver) = socket.split();
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Echo消息（实际应该调用endpoint的处理器）
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

