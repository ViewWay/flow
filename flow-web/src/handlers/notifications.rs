use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use flow_api::extension::{ListOptions, ListResult};
use flow_domain::notification::{Notification, NotificationSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::AppState;

/// 列出通知
pub async fn list_notifications(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListResult<Notification>>, StatusCode> {
    let mut options = ListOptions::default();
    
    // 支持按接收者过滤
    if let Some(recipient) = params.get("recipient") {
        use flow_api::extension::query::Condition;
        options.condition = Some(Condition::Equal {
            index_name: "spec.recipient".to_string(),
            value: serde_json::Value::String(recipient.clone()),
        });
    } else if let Some(unread) = params.get("unread") {
        // 支持按未读状态过滤
        if unread == "true" || unread == "false" {
            use flow_api::extension::query::Condition;
            let value = unread == "true";
            options.condition = Some(Condition::Equal {
                index_name: "spec.unread".to_string(),
                value: serde_json::Value::Bool(value),
            });
        }
    }
    
    state.notification_service.list(options).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 获取通知
pub async fn get_notification(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Notification>, StatusCode> {
    state.notification_service.get(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// 创建通知
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub spec: NotificationSpec,
}

pub async fn create_notification(
    State(state): State<AppState>,
    Json(request): Json<CreateNotificationRequest>,
) -> Result<Json<Notification>, StatusCode> {
    use flow_api::extension::Metadata;
    
    // 生成唯一名称（使用时间戳和纳秒）
    let name = format!("notification-{}", 
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    
    let notification = Notification {
        metadata: Metadata::new(name),
        spec: request.spec,
    };
    
    state.notification_service.create(notification).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 更新通知
pub async fn update_notification(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<CreateNotificationRequest>,
) -> Result<Json<Notification>, StatusCode> {
    let mut notification = state.notification_service.get(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    notification.spec = request.spec;
    
    state.notification_service.update(notification).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 删除通知
pub async fn delete_notification(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    state.notification_service.delete(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// 标记通知为已读
pub async fn mark_notification_as_read(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Notification>, StatusCode> {
    state.notification_service.mark_as_read(&name).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 标记所有通知为已读
#[derive(Debug, Deserialize)]
pub struct MarkAllReadRequest {
    pub recipient: String,
}

pub async fn mark_all_notifications_as_read(
    State(state): State<AppState>,
    Json(request): Json<MarkAllReadRequest>,
) -> Result<StatusCode, StatusCode> {
    state.notification_service.mark_all_as_read(&request.recipient).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// 未读通知数量响应
#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub count: u64,
}

/// 获取未读通知数量
pub async fn get_unread_count(
    Path(recipient): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    let count = state.notification_service.get_unread_count(&recipient).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(UnreadCountResponse { count }))
}

