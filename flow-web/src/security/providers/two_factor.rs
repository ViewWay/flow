use async_trait::async_trait;
use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest, AuthenticatedUser};
use flow_service::security::{TotpAuthService, UserService, RoleService};
use flow_infra::security::{TwoFactorAuthCache, SessionService};
use flow_domain::security::User;
use std::sync::Arc;

/// 2FA认证提供者
/// 用于验证TOTP代码，完成2FA认证流程
pub struct TwoFactorAuthProvider {
    totp_auth_service: Arc<dyn TotpAuthService>,
    user_service: Arc<dyn UserService>,
    role_service: Arc<dyn RoleService>,
    two_factor_auth_cache: Arc<dyn TwoFactorAuthCache>,
    session_service: Arc<dyn SessionService>,
}

impl TwoFactorAuthProvider {
    pub fn new(
        totp_auth_service: Arc<dyn TotpAuthService>,
        user_service: Arc<dyn UserService>,
        role_service: Arc<dyn RoleService>,
        two_factor_auth_cache: Arc<dyn TwoFactorAuthCache>,
        session_service: Arc<dyn SessionService>,
    ) -> Self {
        Self {
            totp_auth_service,
            user_service,
            role_service,
            two_factor_auth_cache,
            session_service,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for TwoFactorAuthProvider {
    async fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // 只处理POST请求到/challenges/two-factor/totp路径
        if request.method != "POST" || !request.path.contains("/challenges/two-factor/totp") {
            return Ok(AuthenticationResult::Unauthenticated);
        }

        // 从请求头中获取Session ID
        let session_id = match get_session_id_from_headers(request) {
            Some(id) => id,
            None => {
                return Ok(AuthenticationResult::Failed("Missing session ID".to_string()));
            }
        };

        // 从Session中获取2FA状态
        let two_factor_state = match self.two_factor_auth_cache.get_state(&session_id).await {
            Ok(Some(state)) => state,
            Ok(None) => {
                return Ok(AuthenticationResult::Failed("2FA state not found or expired".to_string()));
            }
            Err(e) => {
                return Err(format!("Failed to get 2FA state: {}", e).into());
            }
        };

        // 从请求中提取TOTP代码
        let code = match extract_totp_code(request) {
            Some(c) => c,
            None => {
                return Ok(AuthenticationResult::Failed("Missing TOTP code".to_string()));
            }
        };

        // 获取用户信息
        let user = match self.user_service.get(&two_factor_state.username).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Ok(AuthenticationResult::Failed("User not found".to_string()));
            }
            Err(e) => {
                return Err(format!("Failed to get user: {}", e).into());
            }
        };

        // 验证TOTP代码
        if let Some(encrypted_secret) = &user.spec.totp_encrypted_secret {
            // 解密TOTP密钥
            let raw_secret = match self.totp_auth_service.decrypt_secret(encrypted_secret) {
                Ok(secret) => secret,
                Err(e) => {
                    return Err(format!("Failed to decrypt TOTP secret: {}", e).into());
                }
            };
            
            // 验证TOTP代码
            if !self.totp_auth_service.validate_totp(&raw_secret, code) {
                return Ok(AuthenticationResult::Failed("Invalid TOTP code".to_string()));
            }
        } else {
            return Ok(AuthenticationResult::Failed("User has no TOTP secret configured".to_string()));
        }

        // 验证成功，删除2FA状态
        let _ = self.two_factor_auth_cache.remove_state(&session_id).await;

        // 创建认证用户
        let authenticated_user = AuthenticatedUser {
            username: two_factor_state.username,
            roles: two_factor_state.roles,
            authorities: vec![], // 可以从role_service获取
        };

        Ok(AuthenticationResult::Authenticated(authenticated_user))
    }

    fn priority(&self) -> u32 {
        10 // 2FA优先级较高，在Form Login之后
    }
}

/// 从请求中提取TOTP代码
fn extract_totp_code(request: &AuthRequest) -> Option<u32> {
    // 尝试从JSON body中解析
    if let Some(body_bytes) = &request.body {
        // 将Vec<u8>转换为字符串
        let body_str = match String::from_utf8(body_bytes.clone()) {
            Ok(s) => s,
            Err(_) => return None,
        };
        
        // 尝试解析JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
            // 尝试从JSON中获取code字段
            if let Some(code_value) = json.get("code") {
                if let Some(code_str) = code_value.as_str() {
                    if let Ok(code) = code_str.parse::<u32>() {
                        return Some(code);
                    }
                } else if let Some(code) = code_value.as_u64() {
                    return Some(code as u32);
                }
            }
        }
        
        // 尝试解析表单数据 (application/x-www-form-urlencoded)
        // 格式: code=123456
        if let Some(code_start) = body_str.find("code=") {
            let code_part = &body_str[code_start + 5..];
            let code_str = code_part.split('&').next().unwrap_or(code_part);
            if let Ok(code) = code_str.parse::<u32>() {
                return Some(code);
            }
        }
    }
    
    None
}

/// 从请求头中获取Session ID
fn get_session_id_from_headers(request: &AuthRequest) -> Option<String> {
    // 从Cookie头中提取SESSION cookie
    if let Some(cookie_header) = request.headers.get("cookie") {
        for cookie in cookie_header.split(';') {
            let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
            if parts.len() == 2 && parts[0].trim() == "SESSION" {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    
    None
}

