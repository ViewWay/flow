use async_trait::async_trait;
use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest};
use flow_api::extension::ExtensionClient;
use flow_domain::security::PersonalAccessToken;
use flow_infra::security::JwtService;
use flow_infra::extension::ReactiveExtensionClient;
use std::sync::Arc;

/// PAT（Personal Access Token）认证提供者
/// 注意：由于ExtensionClient trait的限制，这里使用具体的ReactiveExtensionClient类型
pub struct PatProvider {
    client: Arc<ReactiveExtensionClient>,
    jwt_service: Arc<JwtService>,
}

impl PatProvider {
    pub fn new(
        client: Arc<ReactiveExtensionClient>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            client,
            jwt_service,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for PatProvider {
    async fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // 提取Bearer Token
        let auth_header = match request.get_header("authorization") {
            Some(header) => header,
            None => return Ok(AuthenticationResult::Unauthenticated),
        };

        if !auth_header.starts_with("Bearer ") {
            return Ok(AuthenticationResult::Unauthenticated);
        }

        let token = &auth_header[7..]; // 跳过"Bearer "

        // 验证JWT
        let claims = match self.jwt_service.verify(token) {
            Ok(claims) => claims,
            Err(_) => return Ok(AuthenticationResult::Failed("Invalid token".to_string())),
        };

        // 检查是否是PAT（必须有pat_name和jti）
        let pat_name = match claims.pat_name {
            Some(name) => name,
            None => return Ok(AuthenticationResult::Unauthenticated),
        };

        let jti = match claims.jti {
            Some(id) => id,
            None => return Ok(AuthenticationResult::Unauthenticated),
        };

        // 查找PAT实体
        let pat = match self.client.fetch::<PersonalAccessToken>(&pat_name).await? {
            Some(pat) => pat,
            None => return Ok(AuthenticationResult::Failed("PAT not found".to_string())),
        };

        // 检查PAT是否有效
        if !pat.is_valid() {
            return Ok(AuthenticationResult::Failed("PAT is revoked or expired".to_string()));
        }

        // 检查Token ID是否匹配
        if !pat.matches_token_id(&jti) {
            return Ok(AuthenticationResult::Failed("Token ID mismatch".to_string()));
        }

        // 获取PAT绑定的角色
        let roles = pat.spec.roles.clone();

        Ok(AuthenticationResult::Authenticated(
            flow_api::security::AuthenticatedUser::new(claims.sub, roles)
        ))
    }

    fn priority(&self) -> u32 {
        5 // PAT优先级较高
    }
}

