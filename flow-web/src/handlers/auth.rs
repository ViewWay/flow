use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_api::security::AuthenticatedUser;
use flow_domain::security::User;
use crate::AppState;
use serde::{Deserialize, Serialize};

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

/// 用户信息（不包含敏感信息）
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub avatar: Option<String>,
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            username: user.metadata.name.clone(),
            display_name: user.spec.display_name.clone(),
            email: user.spec.email.clone(),
            avatar: user.spec.avatar.clone(),
        }
    }
}

/// 登录端点
/// POST /api/v1alpha1/login
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Response, StatusCode> {
    // 查找用户
    let user = match state.user_service.get(&request.username).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // 验证密码
    if let Some(ref password_hash) = user.spec.password {
        match state.password_service.verify(&request.password, password_hash).await {
            Ok(true) => {}
            Ok(false) => return Err(StatusCode::UNAUTHORIZED),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 检查用户是否被禁用
    if user.spec.disabled.unwrap_or(false) {
        return Err(StatusCode::FORBIDDEN);
    }

    // 获取用户角色
    let roles = match state.role_service.get_user_roles(&request.username).await {
        Ok(roles) => roles,
        Err(_) => vec!["authenticated".to_string()],
    };

    // 生成JWT令牌
    let token = match state.jwt_service.generate(request.username.clone()) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // 创建Session（可选）
    let _session_id = match state.session_service.create(
        &AuthenticatedUser::new(request.username.clone(), roles),
        Some(3600),
    ).await {
        Ok(session_id) => Some(session_id),
        Err(_) => None, // Session创建失败不影响登录
    };

    let expires_in = state.jwt_service.expiration();
    let response = LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: UserInfo::from(&user),
    };

    Ok(Json(response).into_response())
}

/// 获取当前用户信息
/// GET /api/v1alpha1/users/-/current
pub async fn get_current_user(
    State(_state): State<AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    // 从请求扩展中获取已认证的用户
    let authenticated_user = match request.extensions().get::<AuthenticatedUser>() {
        Some(user) => user,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // 获取用户详细信息
    let user = match _state.user_service.get(&authenticated_user.username).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(UserInfo::from(&user)).into_response())
}

/// 登出端点
/// POST /api/v1alpha1/logout
pub async fn logout(
    State(_state): State<AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    // 从请求扩展中获取已认证的用户
    let _authenticated_user = match request.extensions().get::<AuthenticatedUser>() {
        Some(user) => user,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // TODO: 如果使用Session，这里应该删除Session
    // 对于JWT，由于是无状态的，只需返回成功即可
    // 客户端应该在本地删除token

    Ok((StatusCode::OK, "Logged out successfully").into_response())
}

