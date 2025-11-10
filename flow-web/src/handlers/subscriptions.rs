use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use flow_api::extension::{ListOptions, ListResult, ExtensionClient};
use flow_domain::notification::{
    Reason, ReasonSpec, ReasonSubject,
    Subscription, SubscriptionSpec, SubscriptionSubscriber, InterestReason,
};
use flow_service::NotificationCenter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::AppState;

/// 列出订阅
pub async fn list_subscriptions(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListResult<Subscription>>, StatusCode> {
    let mut options = ListOptions::default();
    
    // 支持按订阅者过滤
    if let Some(subscriber) = params.get("subscriber") {
        use flow_api::extension::query::Condition;
        options.condition = Some(Condition::Equal {
            index_name: "spec.subscriber.name".to_string(),
            value: serde_json::Value::String(subscriber.clone()),
        });
    }
    
    state.extension_client.list(options).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 获取订阅
pub async fn get_subscription(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Subscription>, StatusCode> {
    state.extension_client.fetch(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// 创建订阅请求
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub subscriber: SubscriptionSubscriber,
    pub reason: InterestReason,
}

/// 创建订阅
pub async fn create_subscription(
    State(state): State<AppState>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Result<Json<Subscription>, StatusCode> {
    state.notification_center.subscribe(
        request.subscriber,
        request.reason,
    ).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 删除订阅
pub async fn delete_subscription(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let subscription = state.extension_client.fetch::<Subscription>(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    state.notification_center.unsubscribe(&subscription.spec.subscriber).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// 通过token取消订阅
pub async fn unsubscribe_by_token(
    Path(name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let token = params.get("token")
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let subscription = state.extension_client.fetch::<Subscription>(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // 验证token
    if subscription.spec.unsubscribe_token != *token {
        return Err(StatusCode::FORBIDDEN);
    }
    
    state.notification_center.unsubscribe(&subscription.spec.subscriber).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// 列出原因
pub async fn list_reasons(
    State(state): State<AppState>,
    Query(_params): Query<HashMap<String, String>>,
) -> Result<Json<ListResult<Reason>>, StatusCode> {
    let options = ListOptions::default();
    state.extension_client.list(options).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 获取原因
pub async fn get_reason(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Reason>, StatusCode> {
    state.extension_client.fetch(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// 创建原因请求
#[derive(Debug, Deserialize)]
pub struct CreateReasonRequest {
    pub spec: ReasonSpec,
}

/// 创建原因
pub async fn create_reason(
    State(state): State<AppState>,
    Json(request): Json<CreateReasonRequest>,
) -> Result<Json<Reason>, StatusCode> {
    use flow_api::extension::Metadata;
    
    // 生成唯一名称（使用时间戳和纳秒）
    let name = format!("reason-{}", 
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    
    let reason = Reason {
        metadata: Metadata::new(name),
        spec: request.spec,
    };
    
    state.extension_client.create(reason).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 删除原因
pub async fn delete_reason(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    state.extension_client.delete::<Reason>(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

