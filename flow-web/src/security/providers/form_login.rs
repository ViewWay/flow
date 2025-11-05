use async_trait::async_trait;
use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest};
use flow_service::security::{UserService, PasswordService};
use flow_infra::security::RateLimiter;
use flow_infra::security::SessionService;
use std::sync::Arc;

/// 表单登录认证提供者
pub struct FormLoginProvider {
    user_service: Arc<dyn UserService>,
    password_service: Arc<dyn PasswordService>,
    rate_limiter: Arc<dyn RateLimiter>,
    session_service: Arc<dyn SessionService>,
    jwt_service: Arc<flow_infra::security::JwtService>,
}

impl FormLoginProvider {
    pub fn new(
        user_service: Arc<dyn UserService>,
        password_service: Arc<dyn PasswordService>,
        rate_limiter: Arc<dyn RateLimiter>,
        session_service: Arc<dyn SessionService>,
        jwt_service: Arc<flow_infra::security::JwtService>,
    ) -> Self {
        Self {
            user_service,
            password_service,
            rate_limiter,
            session_service,
            jwt_service,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for FormLoginProvider {
    async fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // 只处理POST请求到/login路径
        if request.method != "POST" || !request.path.starts_with("/login") {
            return Ok(AuthenticationResult::Unauthenticated);
        }

        // 速率限制检查
        let client_ip_str = request.get_header("x-forwarded-for")
            .or_else(|| request.get_header("x-real-ip"))
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        
        let (allowed, _, _) = self.rate_limiter.check(client_ip_str, 5, 60).await?;
        if !allowed {
            return Ok(AuthenticationResult::Failed("Too many login attempts".to_string()));
        }

        // TODO: 解析表单数据（username和password）
        // 这里简化处理，实际应该从request.body中解析表单数据
        // 或者从headers中获取（如果已经在其他地方解析过）

        // 查找用户并验证密码的逻辑与BasicAuthProvider类似
        // ...

        Ok(AuthenticationResult::Unauthenticated)
    }

    fn priority(&self) -> u32 {
        20 // Form Login优先级中等
    }
}

