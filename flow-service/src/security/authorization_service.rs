use async_trait::async_trait;
use flow_api::security::{AuthorizationManager, AuthorizationDecision, AuthenticatedUser, RequestInfo};
use crate::security::RoleService;
use std::sync::Arc;

/// 默认授权管理器实现（RBAC）
pub struct DefaultAuthorizationManager {
    role_service: Arc<dyn RoleService>,
}

impl DefaultAuthorizationManager {
    pub fn new(role_service: Arc<dyn RoleService>) -> Self {
        Self { role_service }
    }
}

#[async_trait]
impl AuthorizationManager for DefaultAuthorizationManager {
    async fn check(
        &self,
        user: &AuthenticatedUser,
        request_info: &RequestInfo,
    ) -> Result<AuthorizationDecision, Box<dyn std::error::Error + Send + Sync>> {
        // 检查用户空间权限
        if let Some(ref userspace) = request_info.userspace {
            if user.username != *userspace {
                return Ok(AuthorizationDecision::deny(Some(
                    format!("User {} does not own userspace {}", user.username, userspace)
                )));
            }
        }

        // 获取用户的所有角色（包括依赖角色）
        let roles = self.role_service.list_dependencies(&user.roles).await?;

        // 遍历角色规则，查找匹配的规则
        for role in roles {
            if role.rules.is_empty() {
                continue;
            }

            for rule in &role.rules {
                if rule.matches_request(
                    &request_info.verb,
                    request_info.api_group.as_deref(),
                    request_info.resource.as_deref(),
                    request_info.name.as_deref(),
                    request_info.subresource.as_deref(),
                ) {
                    return Ok(AuthorizationDecision::allow(Some(
                        format!("RBAC: allowed by role {}", role.metadata.name)
                    )));
                }
            }
        }

        // 没有找到匹配的规则，拒绝访问
        Ok(AuthorizationDecision::deny(Some(
            "No matching policy rule found".to_string()
        )))
    }
}

