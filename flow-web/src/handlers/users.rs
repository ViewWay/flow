use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_api::extension::ListOptions;
use flow_domain::security::User;
use crate::AppState;
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub password: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub disabled: Option<bool>,
}

/// 用户列表响应
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub items: Vec<User>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 创建用户
/// POST /api/v1alpha1/users
pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Response, StatusCode> {
    use flow_api::extension::Metadata;
    use flow_domain::security::UserSpec;

    // 检查用户名是否已存在
    if let Ok(Some(_)) = state.user_service.get(&request.username).await {
        return Err(StatusCode::CONFLICT);
    }

    // 检查邮箱是否已存在
    if let Ok(Some(_)) = state.user_service.get_by_email(&request.email).await {
        return Err(StatusCode::CONFLICT);
    }

    // 加密密码
    let password_hash = match state.password_service.hash(&request.password).await {
        Ok(hash) => Some(hash),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // 创建用户
    let user = User {
        metadata: Metadata::new(request.username.clone()),
        spec: UserSpec {
            display_name: request.display_name,
            avatar: request.avatar,
            email: request.email,
            email_verified: Some(false),
            phone: None,
            password: password_hash,
            bio: request.bio,
            registered_at: Some(Utc::now()),
            two_factor_auth_enabled: Some(false),
            totp_encrypted_secret: None,
            disabled: Some(false),
            login_history_limit: Some(10),
        },
        status: None,
    };

    match state.user_service.create(user).await {
        Ok(user) => Ok(Json(user).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取用户
/// GET /api/v1alpha1/users/{name}
pub async fn get_user(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.user_service.get(&name).await {
        Ok(Some(user)) => Ok(Json(user).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 更新用户
/// PUT /api/v1alpha1/users/{name}
pub async fn update_user(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Response, StatusCode> {
    // 获取现有用户
    let mut user = match state.user_service.get(&name).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // 更新字段
    if let Some(display_name) = request.display_name {
        user.spec.display_name = display_name;
    }
    if let Some(email) = request.email {
        user.spec.email = email;
    }
    if let Some(avatar) = request.avatar {
        user.spec.avatar = Some(avatar);
    }
    if let Some(bio) = request.bio {
        user.spec.bio = Some(bio);
    }
    if let Some(disabled) = request.disabled {
        user.spec.disabled = Some(disabled);
    }

    match state.user_service.update(user).await {
        Ok(user) => Ok(Json(user).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除用户
/// DELETE /api/v1alpha1/users/{name}
pub async fn delete_user(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match state.user_service.delete(&name).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, "").into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出用户
/// GET /api/v1alpha1/users
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListOptions>,
) -> Result<Response, StatusCode> {
    match state.user_service.list(params).await {
        Ok(result) => {
            let response = UserListResponse {
                items: result.items,
                total: result.total,
                page: result.page as u64,
                size: result.size as u64,
            };
            Ok(Json(response).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

