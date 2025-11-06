use async_trait::async_trait;
use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest};
use flow_service::security::{UserService, PasswordService, RoleService};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::sync::Arc;

/// HTTP Basic认证提供者
pub struct BasicAuthProvider {
    user_service: Arc<dyn UserService>,
    password_service: Arc<dyn PasswordService>,
    role_service: Arc<dyn RoleService>,
}

impl BasicAuthProvider {
    pub fn new(
        user_service: Arc<dyn UserService>,
        password_service: Arc<dyn PasswordService>,
        role_service: Arc<dyn RoleService>,
    ) -> Self {
        Self {
            user_service,
            password_service,
            role_service,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for BasicAuthProvider {
    async fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // 提取Authorization头
        let auth_header = match request.get_header("authorization") {
            Some(header) => header,
            None => return Ok(AuthenticationResult::Unauthenticated),
        };

        // 检查是否是Basic认证
        if !auth_header.starts_with("Basic ") {
            return Ok(AuthenticationResult::Unauthenticated);
        }

        // 解码Base64
        let encoded = &auth_header[6..]; // 跳过"Basic "
        let decoded = STANDARD.decode(encoded)
            .map_err(|e| format!("Base64 decode error: {}", e))?;
        
        let credentials = String::from_utf8(decoded)
            .map_err(|e| format!("UTF-8 decode error: {}", e))?;

        // 解析用户名和密码
        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Ok(AuthenticationResult::Failed("Invalid credentials format".to_string()));
        }

        let username = parts[0];
        let password = parts[1];

        // 查找用户
        let user = match self.user_service.get(username).await? {
            Some(user) => user,
            None => return Ok(AuthenticationResult::Failed("User not found".to_string())),
        };

        // 验证密码
        if let Some(ref password_hash) = user.spec.password {
            let verified = self.password_service.verify(password, password_hash).await?;
            if !verified {
                return Ok(AuthenticationResult::Failed("Invalid password".to_string()));
            }
        } else {
            return Ok(AuthenticationResult::Failed("User has no password".to_string()));
        }

        // 检查用户是否被禁用
        if user.spec.disabled.unwrap_or(false) {
            return Ok(AuthenticationResult::Failed("User is disabled".to_string()));
        }

        // 从RoleService获取用户角色
        let roles = match self.role_service.get_user_roles(username).await {
            Ok(roles) => roles,
            Err(e) => {
                // 如果获取角色失败，记录日志但继续认证（至少允许authenticated角色）
                eprintln!("Failed to get user roles: {}", e);
                vec!["authenticated".to_string()]
            }
        };

        Ok(AuthenticationResult::Authenticated(
            flow_api::security::AuthenticatedUser::new(username.to_string(), roles)
        ))
    }

    fn priority(&self) -> u32 {
        10 // Basic Auth优先级较低
    }
}

