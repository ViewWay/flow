use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::body::Bytes;
use flow_api::extension::ListOptions;
use crate::AppState;
use serde_json::json;

/// 列出主题
pub async fn list_themes(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.theme_service.list_themes(ListOptions::default()).await {
        Ok(themes) => (StatusCode::OK, Json(json!({"items": themes}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to list themes: {}", e)})),
        ).into_response(),
    }
}

/// 获取主题
pub async fn get_theme(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.theme_service.get_theme(&name).await {
        Ok(Some(theme)) => (StatusCode::OK, Json(theme)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Theme not found: {}", name)})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to get theme: {}", e)})),
        ).into_response(),
    }
}

/// 激活主题
pub async fn activate_theme(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.theme_service.set_active_theme(&name).await {
        Ok(_) => {
            // 清除模板引擎缓存
            state.template_engine_manager.clear_cache(&name).await;
            
            // 获取激活后的主题信息
            match state.theme_service.get_theme(&name).await {
                Ok(Some(theme)) => (StatusCode::OK, Json(theme)).into_response(),
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": format!("Theme not found: {}", name)})),
                ).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Failed to get theme: {}", e)})),
                ).into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to activate theme: {}", e)})),
        ).into_response(),
    }
}

/// 安装主题
pub async fn install_theme(
    State(state): State<AppState>,
    body: Bytes,
) -> impl IntoResponse {
    // 将请求体转换为Vec<u8>
    let content = body.to_vec();
    
    match state.theme_service.install_theme(content).await {
        Ok(theme) => {
            // 清除模板引擎缓存（如果主题已存在）
            state.template_engine_manager.clear_cache(&theme.metadata.name).await;
            (StatusCode::OK, Json(theme)).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to install theme: {}", e)})),
        ).into_response(),
    }
}

/// 升级主题
pub async fn upgrade_theme(
    Path(name): Path<String>,
    State(state): State<AppState>,
    body: Bytes,
) -> impl IntoResponse {
    // 将请求体转换为Vec<u8>
    let content = body.to_vec();
    
    match state.theme_service.upgrade_theme(&name, content).await {
        Ok(theme) => {
            // 清除模板引擎缓存
            state.template_engine_manager.clear_cache(&name).await;
            (StatusCode::OK, Json(theme)).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to upgrade theme: {}", e)})),
        ).into_response(),
    }
}

/// 重新加载主题
pub async fn reload_theme(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.theme_service.reload_theme(&name).await {
        Ok(theme) => {
            // 清除模板引擎缓存
            state.template_engine_manager.clear_cache(&name).await;
            (StatusCode::OK, Json(theme)).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to reload theme: {}", e)})),
        ).into_response(),
    }
}

