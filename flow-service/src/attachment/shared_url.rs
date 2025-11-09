use flow_domain::attachment::Attachment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use anyhow::Result;

/// 共享URL信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedUrl {
    /// 共享token
    pub token: String,
    /// 附件名称
    pub attachment_name: String,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 共享URL服务trait
pub trait SharedUrlService: Send + Sync {
    /// 生成共享URL
    fn generate_shared_url(&self, attachment_name: &str, expires_in_hours: Option<u32>) -> Result<SharedUrl>;
    
    /// 验证共享URL token
    fn validate_token(&self, token: &str) -> Result<Option<String>>;
    
    /// 删除共享URL
    fn revoke_shared_url(&self, token: &str) -> Result<()>;
    
    /// 获取附件的所有共享URL
    fn get_shared_urls(&self, attachment_name: &str) -> Result<Vec<SharedUrl>>;
}

/// 默认共享URL服务实现（内存存储）
pub struct DefaultSharedUrlService {
    /// 存储共享URL（token -> SharedUrl）
    urls: Arc<RwLock<HashMap<String, SharedUrl>>>,
    /// 存储附件的共享URL列表（attachment_name -> Vec<token>）
    attachment_urls: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl DefaultSharedUrlService {
    pub fn new() -> Self {
        Self {
            urls: Arc::new(RwLock::new(HashMap::new())),
            attachment_urls: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 清理过期的共享URL
    async fn cleanup_expired(&self) {
        let mut urls = self.urls.write().await;
        let mut attachment_urls = self.attachment_urls.write().await;
        
        let now = Utc::now();
        let expired_tokens: Vec<String> = urls.iter()
            .filter(|(_, url)| url.expires_at < now)
            .map(|(token, _)| token.clone())
            .collect();
        
        for token in expired_tokens {
            if let Some(url) = urls.remove(&token) {
                // 从附件的URL列表中移除
                if let Some(tokens) = attachment_urls.get_mut(&url.attachment_name) {
                    tokens.retain(|t| t != &token);
                    if tokens.is_empty() {
                        attachment_urls.remove(&url.attachment_name);
                    }
                }
            }
        }
    }
}

impl Default for DefaultSharedUrlService {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedUrlService for DefaultSharedUrlService {
    fn generate_shared_url(&self, attachment_name: &str, expires_in_hours: Option<u32>) -> Result<SharedUrl> {
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_in = Duration::hours(expires_in_hours.unwrap_or(24) as i64);
        let expires_at = now + expires_in;
        
        let shared_url = SharedUrl {
            token: token.clone(),
            attachment_name: attachment_name.to_string(),
            expires_at,
            created_at: now,
        };
        
        // 使用tokio运行时同步执行
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;
        
        rt.block_on(async {
            let mut urls = self.urls.write().await;
            let mut attachment_urls = self.attachment_urls.write().await;
            
            urls.insert(token.clone(), shared_url.clone());
            attachment_urls
                .entry(attachment_name.to_string())
                .or_insert_with(Vec::new)
                .push(token);
            
            // 清理过期URL
            self.cleanup_expired().await;
        });
        
        Ok(shared_url)
    }
    
    fn validate_token(&self, token: &str) -> Result<Option<String>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;
        
        rt.block_on(async {
            let mut urls = self.urls.write().await;
            
            // 清理过期URL
            self.cleanup_expired().await;
            
            if let Some(url) = urls.get(token) {
                if url.expires_at > Utc::now() {
                    Ok(Some(url.attachment_name.clone()))
                } else {
                    // 已过期，删除
                    urls.remove(token);
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
    }
    
    fn revoke_shared_url(&self, token: &str) -> Result<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;
        
        rt.block_on(async {
            let mut urls = self.urls.write().await;
            let mut attachment_urls = self.attachment_urls.write().await;
            
            if let Some(url) = urls.remove(token) {
                // 从附件的URL列表中移除
                if let Some(tokens) = attachment_urls.get_mut(&url.attachment_name) {
                    tokens.retain(|t| t != token);
                    if tokens.is_empty() {
                        attachment_urls.remove(&url.attachment_name);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    fn get_shared_urls(&self, attachment_name: &str) -> Result<Vec<SharedUrl>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;
        
        rt.block_on(async {
            let urls = self.urls.read().await;
            let attachment_urls = self.attachment_urls.read().await;
            
            if let Some(tokens) = attachment_urls.get(attachment_name) {
                let mut result = Vec::new();
                for token in tokens {
                    if let Some(url) = urls.get(token) {
                        // 只返回未过期的URL
                        if url.expires_at > Utc::now() {
                            result.push(url.clone());
                        }
                    }
                }
                Ok(result)
            } else {
                Ok(Vec::new())
            }
        })
    }
}

