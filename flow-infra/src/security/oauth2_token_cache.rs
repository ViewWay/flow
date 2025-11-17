use async_trait::async_trait;
use crate::cache::Cache;
use serde_json;
use std::sync::Arc;
use std::collections::HashMap;

/// OAuth2 token信息
/// 存储在Session中，用于OAuth2认证
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuth2TokenInfo {
    /// Access token
    pub access_token: String,
    /// Registration ID（OAuth2提供者ID）
    pub registration_id: String,
    /// Provider user ID（OAuth2提供者返回的用户ID）
    pub provider_user_id: String,
    /// 用户属性（从OAuth2提供者返回）
    pub attributes: HashMap<String, serde_json::Value>,
}

/// OAuth2 token缓存服务trait
#[async_trait]
pub trait OAuth2TokenCache: Send + Sync {
    /// 保存OAuth2 token到Session
    async fn save_token(&self, session_id: &str, token: OAuth2TokenInfo) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 从Session中获取OAuth2 token
    async fn get_token(&self, session_id: &str) 
        -> Result<Option<OAuth2TokenInfo>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 从Session中删除OAuth2 token
    async fn remove_token(&self, session_id: &str) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 基于Redis的OAuth2 token缓存实现
pub struct RedisOAuth2TokenCache {
    cache: Arc<dyn Cache>,
    token_prefix: String,
    default_ttl: u64,
}

impl RedisOAuth2TokenCache {
    pub fn new(cache: Arc<dyn Cache>, default_ttl: u64) -> Self {
        Self {
            cache,
            token_prefix: "oauth2_token:".to_string(),
            default_ttl,
        }
    }

    fn token_key(&self, session_id: &str) -> String {
        format!("{}{}", self.token_prefix, session_id)
    }
}

#[async_trait]
impl OAuth2TokenCache for RedisOAuth2TokenCache {
    async fn save_token(&self, session_id: &str, token: OAuth2TokenInfo) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = self.token_key(session_id);
        let token_json = serde_json::to_string(&token)
            .map_err(|e| format!("Serialize OAuth2 token error: {}", e))?;
        
        self.cache.set(&key, &token_json, Some(self.default_ttl)).await?;
        Ok(())
    }

    async fn get_token(&self, session_id: &str) 
        -> Result<Option<OAuth2TokenInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let key = self.token_key(session_id);
        
        match self.cache.get(&key).await? {
            Some(token_json) => {
                let token: OAuth2TokenInfo = serde_json::from_str(&token_json)
                    .map_err(|e| format!("Deserialize OAuth2 token error: {}", e))?;
                Ok(Some(token))
            }
            None => Ok(None),
        }
    }

    async fn remove_token(&self, session_id: &str) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = self.token_key(session_id);
        self.cache.delete(&key).await?;
        Ok(())
    }
}

