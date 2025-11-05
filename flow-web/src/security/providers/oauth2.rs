use async_trait::async_trait;
use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest};

/// OAuth2认证提供者（占位实现）
pub struct OAuth2Provider {
    // TODO: 实现OAuth2客户端
}

impl OAuth2Provider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl AuthenticationProvider for OAuth2Provider {
    async fn authenticate(
        &self,
        _request: &AuthRequest,
    ) -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现OAuth2认证流程
        Ok(AuthenticationResult::Unauthenticated)
    }

    fn priority(&self) -> u32 {
        15 // OAuth2优先级中等
    }
}

impl Default for OAuth2Provider {
    fn default() -> Self {
        Self::new()
    }
}

