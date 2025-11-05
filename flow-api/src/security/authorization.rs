use async_trait::async_trait;
use crate::security::{AuthenticatedUser, RequestInfo};

/// 授权决策
#[derive(Debug, Clone)]
pub struct AuthorizationDecision {
    /// 是否允许访问
    pub allowed: bool,
    /// 决策原因（用于日志和调试）
    pub reason: Option<String>,
}

impl AuthorizationDecision {
    pub fn allow(reason: Option<String>) -> Self {
        Self {
            allowed: true,
            reason,
        }
    }

    pub fn deny(reason: Option<String>) -> Self {
        Self {
            allowed: false,
            reason,
        }
    }
}

/// 授权管理器trait
/// 实现RBAC（基于角色的访问控制）授权逻辑
#[async_trait]
pub trait AuthorizationManager: Send + Sync {
    /// 检查用户是否有权限访问指定的请求
    /// 
    /// # 参数
    /// - `user`: 认证后的用户信息
    /// - `request_info`: 从HTTP请求解析的资源信息
    /// 
    /// # 返回
    /// - `Ok(AuthorizationDecision)`: 授权决策
    /// - `Err`: 授权检查过程中的错误
    async fn check(
        &self,
        user: &AuthenticatedUser,
        request_info: &RequestInfo,
    ) -> Result<AuthorizationDecision, Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_decision() {
        let allow_decision = AuthorizationDecision::allow(Some("Allowed by rule".to_string()));
        assert!(allow_decision.allowed);
        assert_eq!(allow_decision.reason, Some("Allowed by rule".to_string()));

        let deny_decision = AuthorizationDecision::deny(Some("Denied: no permission".to_string()));
        assert!(!deny_decision.allowed);
        assert_eq!(deny_decision.reason, Some("Denied: no permission".to_string()));
    }
}

