use async_trait::async_trait;
use crate::cache::Cache;
use serde_json;
use std::sync::Arc;

/// 2FA认证状态信息
/// 存储在Session中，用于2FA验证流程
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TwoFactorAuthState {
    /// 用户名（已通过密码验证）
    pub username: String,
    /// 用户角色（已获取）
    pub roles: Vec<String>,
    /// 创建时间戳（用于过期检查）
    pub created_at: i64,
}

/// 2FA状态缓存服务trait
#[async_trait]
pub trait TwoFactorAuthCache: Send + Sync {
    /// 保存2FA状态到Session
    async fn save_state(&self, session_id: &str, state: TwoFactorAuthState, ttl: Option<u64>) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 从Session中获取2FA状态
    async fn get_state(&self, session_id: &str) 
        -> Result<Option<TwoFactorAuthState>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 从Session中删除2FA状态
    async fn remove_state(&self, session_id: &str) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 基于Redis的2FA状态缓存实现
pub struct RedisTwoFactorAuthCache {
    cache: Arc<dyn Cache>,
    state_prefix: String,
    default_ttl: u64,
}

impl RedisTwoFactorAuthCache {
    pub fn new(cache: Arc<dyn Cache>, default_ttl: u64) -> Self {
        Self {
            cache,
            state_prefix: "2fa_state:".to_string(),
            default_ttl,
        }
    }

    fn state_key(&self, session_id: &str) -> String {
        format!("{}{}", self.state_prefix, session_id)
    }
}

#[async_trait]
impl TwoFactorAuthCache for RedisTwoFactorAuthCache {
    async fn save_state(&self, session_id: &str, state: TwoFactorAuthState, ttl: Option<u64>) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = self.state_key(session_id);
        
        let state_json = serde_json::to_string(&state)
            .map_err(|e| format!("Serialize 2FA state error: {}", e))?;
        
        let ttl = ttl.unwrap_or(self.default_ttl);
        self.cache.set(&key, &state_json, Some(ttl)).await?;
        
        Ok(())
    }

    async fn get_state(&self, session_id: &str) 
        -> Result<Option<TwoFactorAuthState>, Box<dyn std::error::Error + Send + Sync>> {
        let key = self.state_key(session_id);
        
        match self.cache.get(&key).await? {
            Some(state_json) => {
                let state: TwoFactorAuthState = serde_json::from_str(&state_json)
                    .map_err(|e| format!("Deserialize 2FA state error: {}", e))?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    async fn remove_state(&self, session_id: &str) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = self.state_key(session_id);
        self.cache.delete(&key).await?;
        Ok(())
    }
}

