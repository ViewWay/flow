use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest};
use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;

/// 认证服务（整合所有认证提供者）
/// 使用Arc<dyn AuthenticationProvider>以便在异步上下文中安全使用
pub struct AuthService {
    providers: Arc<RwLock<BTreeMap<u32, Arc<dyn AuthenticationProvider>>>>,
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
        // 将Box转换为Arc
        let provider_arc: Arc<dyn AuthenticationProvider> = Arc::from(provider);
        let mut providers = self.providers.write().unwrap();
        providers.insert(priority, provider_arc);
    }

    /// 认证请求
    pub async fn authenticate(&self, request: &AuthRequest) 
        -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // 先获取providers的Arc引用列表，避免在异步上下文中持有锁
        let providers_vec: Vec<(u32, Arc<dyn AuthenticationProvider>)> = {
            let providers = self.providers.read().unwrap();
            // 按优先级顺序收集所有provider的Arc引用
            providers.iter()
                .map(|(priority, provider)| (*priority, Arc::clone(provider)))
                .collect()
        };
        
        // 现在锁已经释放，可以安全地在异步上下文中使用providers_vec
        // 按优先级顺序尝试各个认证提供者
        for (_priority, provider) in providers_vec.iter() {
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

