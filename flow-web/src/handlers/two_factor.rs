use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use flow_service::security::TotpAuthService;
use serde::{Deserialize, Serialize};
use crate::{AppState, extractors::CurrentUser};

/// 2FA设置响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthSettings {
    pub enabled: bool,
    pub email_verified: bool,
    pub totp_configured: bool,
}

/// 密码请求
#[derive(Debug, Deserialize)]
pub struct PasswordRequest {
    pub password: String,
}

/// TOTP配置请求
#[derive(Debug, Deserialize)]
pub struct TotpRequest {
    pub password: String,
    pub secret: String,
    pub code: String,
}

/// TOTP认证链接响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TotpAuthLinkResponse {
    pub auth_link: String,
    pub raw_secret: String,
}

/// 获取2FA设置
/// GET /apis/uc.api.security.halo.run/v1alpha1/authentications/two-factor/settings
pub async fn get_two_factor_settings(
    State(app_state): State<AppState>,
    CurrentUser(username): CurrentUser,
) -> Result<Json<TwoFactorAuthSettings>, StatusCode> {
    // 获取用户信息
    let user = match app_state.user_service.get(&username).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 构建2FA设置
    let settings = TwoFactorAuthSettings {
        enabled: user.spec.two_factor_auth_enabled.unwrap_or(false),
        email_verified: user.spec.email_verified.unwrap_or(false),
        totp_configured: user.spec.totp_encrypted_secret.is_some(),
    };
    
    Ok(Json(settings))
}

/// 启用2FA
/// PUT /apis/uc.api.security.halo.run/v1alpha1/authentications/two-factor/settings/enabled
pub async fn enable_two_factor(
    State(app_state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Json(request): Json<PasswordRequest>,
) -> Result<Json<TwoFactorAuthSettings>, StatusCode> {
    // 获取用户信息
    let mut user = match app_state.user_service.get(&username).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 验证密码
    if let Some(password_hash) = &user.spec.password {
        if !app_state.password_service.verify(&request.password, password_hash).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 检查是否已配置TOTP
    if user.spec.totp_encrypted_secret.is_none() {
        return Err(StatusCode::BAD_REQUEST); // 需要先配置TOTP
    }
    
    // 更新用户的two_factor_auth_enabled字段
    user.spec.two_factor_auth_enabled = Some(true);
    
    // 保存用户
    let updated_user = match app_state.user_service.update(user).await {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 返回更新后的设置
    Ok(Json(TwoFactorAuthSettings {
        enabled: updated_user.spec.two_factor_auth_enabled.unwrap_or(false),
        email_verified: updated_user.spec.email_verified.unwrap_or(false),
        totp_configured: updated_user.spec.totp_encrypted_secret.is_some(),
    }))
}

/// 禁用2FA
/// PUT /apis/uc.api.security.halo.run/v1alpha1/authentications/two-factor/settings/disabled
pub async fn disable_two_factor(
    State(app_state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Json(request): Json<PasswordRequest>,
) -> Result<Json<TwoFactorAuthSettings>, StatusCode> {
    // 获取用户信息
    let mut user = match app_state.user_service.get(&username).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 验证密码
    if let Some(password_hash) = &user.spec.password {
        if !app_state.password_service.verify(&request.password, password_hash).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 更新用户的two_factor_auth_enabled字段为false
    user.spec.two_factor_auth_enabled = Some(false);
    
    // 保存用户
    let updated_user = match app_state.user_service.update(user).await {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 返回更新后的设置
    Ok(Json(TwoFactorAuthSettings {
        enabled: updated_user.spec.two_factor_auth_enabled.unwrap_or(false),
        email_verified: updated_user.spec.email_verified.unwrap_or(false),
        totp_configured: updated_user.spec.totp_encrypted_secret.is_some(),
    }))
}

/// 配置TOTP
/// POST /apis/uc.api.security.halo.run/v1alpha1/authentications/two-factor/totp
pub async fn configure_totp(
    State(app_state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Json(request): Json<TotpRequest>,
) -> Result<Json<TwoFactorAuthSettings>, StatusCode> {
    // 获取用户信息
    let mut user = match app_state.user_service.get(&username).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 验证密码
    if let Some(password_hash) = &user.spec.password {
        if !app_state.password_service.verify(&request.password, password_hash).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 解析TOTP代码
    let code = request.code.parse::<u32>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // 验证TOTP代码
    if !app_state.totp_auth_service.validate_totp(&request.secret, code) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 加密TOTP密钥
    let encrypted_secret = app_state.totp_auth_service.encrypt_secret(&request.secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 保存加密后的密钥到用户
    user.spec.totp_encrypted_secret = Some(encrypted_secret);
    
    // 保存用户
    let updated_user = match app_state.user_service.update(user).await {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 返回更新后的设置
    Ok(Json(TwoFactorAuthSettings {
        enabled: updated_user.spec.two_factor_auth_enabled.unwrap_or(false),
        email_verified: updated_user.spec.email_verified.unwrap_or(false),
        totp_configured: updated_user.spec.totp_encrypted_secret.is_some(),
    }))
}

/// 删除TOTP
/// DELETE /apis/uc.api.security.halo.run/v1alpha1/authentications/two-factor/totp/-
pub async fn delete_totp(
    State(app_state): State<AppState>,
    CurrentUser(username): CurrentUser,
    Json(request): Json<PasswordRequest>,
) -> Result<Json<TwoFactorAuthSettings>, StatusCode> {
    // 获取用户信息
    let mut user = match app_state.user_service.get(&username).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 验证密码
    if let Some(password_hash) = &user.spec.password {
        if !app_state.password_service.verify(&request.password, password_hash).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 清除用户的totp_encrypted_secret字段
    user.spec.totp_encrypted_secret = None;
    // 同时禁用2FA
    user.spec.two_factor_auth_enabled = Some(false);
    
    // 保存用户
    let updated_user = match app_state.user_service.update(user).await {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 返回更新后的设置
    Ok(Json(TwoFactorAuthSettings {
        enabled: updated_user.spec.two_factor_auth_enabled.unwrap_or(false),
        email_verified: updated_user.spec.email_verified.unwrap_or(false),
        totp_configured: updated_user.spec.totp_encrypted_secret.is_some(),
    }))
}

/// 获取TOTP认证链接
/// GET /apis/uc.api.security.halo.run/v1alpha1/authentications/two-factor/totp/auth-link
pub async fn get_totp_auth_link(
    State(app_state): State<AppState>,
    CurrentUser(username): CurrentUser,
) -> Result<Json<TotpAuthLinkResponse>, StatusCode> {
    // 获取用户信息
    let user = match app_state.user_service.get(&username).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // 生成TOTP密钥
    let raw_secret = app_state.totp_auth_service.generate_totp_secret();
    
    // 构建认证链接（otpauth://totp/...）
    // 格式: otpauth://totp/{issuer}:{account}?secret={secret}&issuer={issuer}
    let issuer = "Flow"; // TODO: 从配置中获取
    let account = format!("{}@{}", username, user.spec.email);
    let auth_link = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        issuer, account, raw_secret, issuer
    );
    
    Ok(Json(TotpAuthLinkResponse {
        auth_link,
        raw_secret,
    }))
}

