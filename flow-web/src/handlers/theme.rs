use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_domain::theme::Theme;
use flow_service::theme::ThemeService;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 列出主题
pub async fn list_themes(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // TODO: 从AppState获取theme_service
    // let themes = state.theme_service.list_themes(Default::default()).await?;
    (StatusCode::OK, Json(serde_json::json!({"items": []})))
}

/// 获取主题
pub async fn get_theme(
    Path(name): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // TODO: 实现获取主题逻辑
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"error": "Not implemented"})))
}

/// 激活主题
pub async fn activate_theme(
    Path(name): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // TODO: 实现激活主题逻辑
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"error": "Not implemented"})))
}

