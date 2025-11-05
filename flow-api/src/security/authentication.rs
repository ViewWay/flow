use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 认证后的用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub username: String,
    pub roles: Vec<String>,
    pub authorities: Vec<String>,
}

impl AuthenticatedUser {
    pub fn new(username: String, roles: Vec<String>) -> Self {
        Self {
            username,
            roles: roles.clone(),
            authorities: roles.into_iter()
                .map(|r| format!("ROLE_{}", r))
                .collect(),
        }
    }

    /// 检查用户是否具有指定的角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// 检查用户是否具有指定的权限
    pub fn has_authority(&self, authority: &str) -> bool {
        self.authorities.contains(&authority.to_string())
    }
}

/// 认证结果
#[derive(Debug, Clone)]
pub enum AuthenticationResult {
    /// 认证成功
    Authenticated(AuthenticatedUser),
    /// 未认证（没有提供凭证）
    Unauthenticated,
    /// 认证失败（凭证无效）
    Failed(String),
}

/// 请求信息（用于认证）
/// 包含从HTTP请求中提取的必要信息
#[derive(Debug, Clone)]
pub struct AuthRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl AuthRequest {
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    pub fn get_cookie(&self, name: &str) -> Option<String> {
        // Cookie头格式：Cookie: name1=value1; name2=value2
        self.headers.get("cookie")
            .and_then(|cookie_header| {
                cookie_header
                    .split(';')
                    .find_map(|pair| {
                        let mut parts = pair.split('=');
                        let key = parts.next()?.trim();
                        let value = parts.next()?.trim();
                        if key == name {
                            Some(value.to_string())
                        } else {
                            None
                        }
                    })
            })
    }
}

/// 认证提供者trait
/// 不同的认证方式（Basic Auth、Form Login、PAT等）实现此trait
#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
    /// 尝试从请求中认证用户
    /// 返回AuthenticationResult表示认证结果
    async fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>>;

    /// 返回认证提供者的优先级
    /// 数字越小优先级越高，认证提供者链会按优先级顺序尝试
    fn priority(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user() {
        let user = AuthenticatedUser::new(
            "test-user".to_string(),
            vec!["admin".to_string(), "user".to_string()],
        );

        assert_eq!(user.username, "test-user");
        assert!(user.has_role("admin"));
        assert!(user.has_role("user"));
        assert!(!user.has_role("guest"));
        assert!(user.has_authority("ROLE_admin"));
        assert!(user.has_authority("ROLE_user"));
    }
}

