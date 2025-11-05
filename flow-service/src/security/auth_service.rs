use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest};
use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;

/// 认证服务（整合所有认证提供者）
pub struct AuthService {
    providers: Arc<RwLock<BTreeMap<u32, Box<dyn AuthenticationProvider>>>>,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// 添加认证提供者
    pub fn add_provider(&self, provider: Box<dyn AuthenticationProvider>) {
        let priority = provider.priority();
        let mut providers = self.providers.write().unwrap();
        providers.insert(priority, provider);
    }

    /// 认证请求
    pub async fn authenticate(&self, request: &AuthRequest) 
        -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // 按优先级顺序尝试各个认证提供者
        let providers = self.providers.read().unwrap();
        for (_priority, provider) in providers.iter() {
            match provider.authenticate(request).await {
                Ok(AuthenticationResult::Authenticated(user)) => {
                    return Ok(AuthenticationResult::Authenticated(user));
                }
                Ok(AuthenticationResult::Unauthenticated) => {
                    // 继续尝试下一个提供者
                    continue;
                }
                Ok(AuthenticationResult::Failed(msg)) => {
                    // 认证失败，返回错误
                    return Ok(AuthenticationResult::Failed(msg));
                }
                Err(e) => {
                    // 提供者错误，记录日志但继续尝试
                    eprintln!("Authentication provider error: {}", e);
                    continue;
                }
            }
        }
        
        // 所有提供者都未认证
        Ok(AuthenticationResult::Unauthenticated)
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

