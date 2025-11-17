use async_trait::async_trait;
use crate::cache::Cache;
use std::sync::Arc;

/// OAuth2 State token缓存服务trait
/// 用于CSRF保护
#[async_trait]
pub trait OAuth2StateCache: Send + Sync {
    /// 保存state token
    async fn save_state(&self, session_id: &str, state: &str, registration_id: &str, ttl: Option<u64>) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取并验证state token
    async fn get_and_verify_state(&self, session_id: &str, state: &str, registration_id: &str) 
        -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 删除state token
    async fn remove_state(&self, session_id: &str) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 基于Redis的OAuth2 state token缓存实现
pub struct RedisOAuth2StateCache {
    cache: Arc<dyn Cache>,
    default_ttl: u64,
    prefix: String,
}

impl RedisOAuth2StateCache {
    pub fn new(cache: Arc<dyn Cache>, default_ttl: u64) -> Self {
        Self {
            cache,
            default_ttl,
            prefix: "oauth2_state:".to_string(),
        }
    }

    fn key(&self, session_id: &str) -> String {
        format!("{}{}", self.prefix, session_id)
    }
}

#[async_trait]
impl OAuth2StateCache for RedisOAuth2StateCache {
    async fn save_state(&self, session_id: &str, state: &str, registration_id: &str, ttl: Option<u64>) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use serde_json;
        
        // 存储state和registration_id的映射
        let state_data = serde_json::json!({
            "state": state,
            "registration_id": registration_id,
        });
        
        let key = self.key(session_id);
        let state_json = serde_json::to_string(&state_data)?;
        let ttl = ttl.unwrap_or(self.default_ttl);
        
        self.cache.set(&key, &state_json, Some(ttl)).await?;
        Ok(())
    }

    async fn get_and_verify_state(&self, session_id: &str, state: &str, registration_id: &str) 
        -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use serde_json;
        
        let key = self.key(session_id);
        match self.cache.get(&key).await? {
            Some(state_json) => {
                let state_data: serde_json::Value = serde_json::from_str(&state_json)?;
                
                // 验证state和registration_id是否匹配
                let stored_state = state_data.get("state")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing state in cached data")?;
                
                let stored_registration_id = state_data.get("registration_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing registration_id in cached data")?;
                
                // 验证state和registration_id是否匹配
                if stored_state == state && stored_registration_id == registration_id {
                    // 验证成功后删除state token（一次性使用）
                    self.cache.delete(&key).await?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    async fn remove_state(&self, session_id: &str) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = self.key(session_id);
        self.cache.delete(&key).await?;
        Ok(())
    }
}

