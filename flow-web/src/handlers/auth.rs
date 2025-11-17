use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
    Json,
};
use flow_api::security::AuthenticatedUser;
use flow_domain::security::User;
use flow_infra::security::TwoFactorAuthState;
use crate::AppState;
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    /// TOTP代码（如果用户启用了2FA）
    #[serde(default)]
    pub totp_code: Option<String>,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

/// 需要2FA验证的响应
#[derive(Debug, Serialize)]
pub struct RequiresTwoFactorResponse {
    pub requires_two_factor: bool,
    pub message: String,
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
    headers: HeaderMap,
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

    // 检查用户是否启用了2FA
    if user.spec.two_factor_auth_enabled.unwrap_or(false) {
        // 用户启用了2FA，需要验证TOTP代码
        // 检查请求中是否包含TOTP代码
        if let Some(totp_code) = request.totp_code.as_ref() {
            // 验证TOTP代码
            if let Some(encrypted_secret) = &user.spec.totp_encrypted_secret {
                // 解密TOTP密钥
                let raw_secret = match state.totp_auth_service.decrypt_secret(encrypted_secret) {
                    Ok(secret) => secret,
                    Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                };
                
                // 解析TOTP代码
                let code = match totp_code.parse::<u32>() {
                    Ok(c) => c,
                    Err(_) => return Err(StatusCode::BAD_REQUEST),
                };
                
                // 验证TOTP代码
                if !state.totp_auth_service.validate_totp(&raw_secret, code) {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::BAD_REQUEST); // 用户启用了2FA但没有配置TOTP
            }
        } else {
            // 没有提供TOTP代码，需要2FA验证
            // 获取用户角色
            let roles = match state.role_service.get_user_roles(&request.username).await {
                Ok(roles) => roles,
                Err(_) => vec!["authenticated".to_string()],
            };
            
            // 创建临时Session用于存储2FA状态
            let anonymous_user = AuthenticatedUser {
                username: "anonymous".to_string(),
                roles: vec![],
                authorities: vec![],
            };
            
            let session_id = match state.session_service.create(&anonymous_user, Some(300)).await {
                Ok(id) => id,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };
            
            // 保存2FA状态到cache
            let two_factor_state = TwoFactorAuthState {
                username: request.username.clone(),
                roles: roles.clone(),
                created_at: Utc::now().timestamp(),
            };
            
            if let Err(_) = state.two_factor_auth_cache.save_state(&session_id, two_factor_state, Some(300)).await {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            
            // 返回需要2FA验证的响应
            let response = RequiresTwoFactorResponse {
                requires_two_factor: true,
                message: "Two-factor authentication required".to_string(),
            };
            
            // 设置Session Cookie并返回响应
            use axum::http::header::{HeaderValue, SET_COOKIE};
            let cookie_value = format!("SESSION={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=300", session_id);
            let mut response = Json(response).into_response();
            response.headers_mut().insert(
                SET_COOKIE,
                HeaderValue::from_str(&cookie_value).unwrap()
            );
            *response.status_mut() = StatusCode::UNAUTHORIZED;
            
            return Ok(response);
        }
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
        &AuthenticatedUser::new(request.username.clone(), roles.clone()),
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
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    // 从请求扩展中获取已认证的用户
    let _authenticated_user = match request.extensions().get::<AuthenticatedUser>() {
        Some(user) => user,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // 从请求头中获取Session ID并删除Session
    use axum::http::HeaderMap;
    let headers = request.headers();
    if let Some(session_id) = get_session_id_from_headers(headers) {
        // 删除Session
        if let Err(e) = state.session_service.delete(&session_id).await {
            tracing::warn!("Failed to delete session during logout: {}", e);
            // 继续执行，不因为Session删除失败而失败
        }
        
        // 删除2FA状态（如果存在）
        let _ = state.two_factor_auth_cache.remove_state(&session_id).await;
        
        // 删除OAuth2 token（如果存在）
        let _ = state.oauth2_token_cache.remove_token(&session_id).await;
    }

    // 对于JWT，由于是无状态的，只需返回成功即可
    // 客户端应该在本地删除token

    Ok((StatusCode::OK, "Logged out successfully").into_response())
}

/// 从请求头中获取Session ID（从Cookie）
fn get_session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    // 从Cookie头中提取SESSION cookie
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0].trim() == "SESSION" {
                    return Some(parts[1].trim().to_string());
                }
            }
        }
    }
    
    None
}

