use async_trait::async_trait;
use flow_api::security::AuthenticatedUser;
use crate::cache::Cache;
use serde_json;
use std::sync::Arc;

/// Session服务trait
#[async_trait]
pub trait SessionService: Send + Sync {
    /// 创建Session
    async fn create(&self, user: &AuthenticatedUser, ttl: Option<u64>) 
        -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取Session
    async fn get(&self, session_id: &str) 
        -> Result<Option<AuthenticatedUser>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 删除Session
    async fn delete(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 刷新Session（延长TTL）
    async fn refresh(&self, session_id: &str, ttl: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 基于Redis的Session服务实现
pub struct RedisSessionService {
    cache: Arc<dyn Cache>,
    default_ttl: u64,
    session_prefix: String,
}

impl RedisSessionService {
    pub fn new(cache: Arc<dyn Cache>, default_ttl: u64) -> Self {
        Self {
            cache,
            default_ttl,
            session_prefix: "session:".to_string(),
        }
    }

    fn session_key(&self, session_id: &str) -> String {
        format!("{}{}", self.session_prefix, session_id)
    }
}

#[async_trait]
impl SessionService for RedisSessionService {
    async fn create(&self, user: &AuthenticatedUser, ttl: Option<u64>) 
        -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use uuid::Uuid;
        
        let session_id = Uuid::new_v4().to_string();
        let key = self.session_key(&session_id);
        
        let user_json = serde_json::to_string(user)
            .map_err(|e| format!("Serialize user error: {}", e))?;
        
        let ttl = ttl.unwrap_or(self.default_ttl);
        self.cache.set(&key, &user_json, Some(ttl)).await?;
        
        Ok(session_id)
    }

    async fn get(&self, session_id: &str) 
        -> Result<Option<AuthenticatedUser>, Box<dyn std::error::Error + Send + Sync>> {
        let key = self.session_key(session_id);
        
        match self.cache.get(&key).await? {
            Some(user_json) => {
                let user: AuthenticatedUser = serde_json::from_str(&user_json)
                    .map_err(|e| format!("Deserialize user error: {}", e))?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = self.session_key(session_id);
        self.cache.delete(&key).await?;
        Ok(())
    }

    async fn refresh(&self, session_id: &str, ttl: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 获取现有session数据
        if let Some(user) = self.get(session_id).await? {
            let key = self.session_key(session_id);
            let user_json = serde_json::to_string(&user)
                .map_err(|e| format!("Serialize user error: {}", e))?;
            // 重新设置TTL
            self.cache.set(&key, &user_json, Some(ttl)).await?;
        }
        Ok(())
    }
}

