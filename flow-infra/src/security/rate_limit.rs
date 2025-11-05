use async_trait::async_trait;
use crate::cache::Cache;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 速率限制器trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// 检查是否允许请求
    /// 返回 (是否允许, 剩余次数, 重置时间戳)
    async fn check(&self, key: &str, limit: u64, window_seconds: u64) 
        -> Result<(bool, u64, u64), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 增加计数
    async fn increment(&self, key: &str, window_seconds: u64) 
        -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;
}

/// 基于Redis的速率限制器实现（滑动窗口算法）
pub struct RedisRateLimiter {
    cache: Arc<dyn Cache>,
    prefix: String,
}

impl RedisRateLimiter {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
            prefix: "rate_limit:".to_string(),
        }
    }

    fn rate_limit_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    fn window_key(&self, base_key: &str, window_start: u64) -> String {
        format!("{}:{}", base_key, window_start)
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check(&self, key: &str, limit: u64, window_seconds: u64) 
        -> Result<(bool, u64, u64), Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let base_key = self.rate_limit_key(key);
        let current_window = now / window_seconds;
        let previous_window = current_window - 1;
        
        // 获取当前窗口和上一个窗口的计数
        let current_key = self.window_key(&base_key, current_window);
        let previous_key = self.window_key(&base_key, previous_window);
        
        let current_count: u64 = self.cache.get(&current_key).await?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        
        let previous_count: u64 = self.cache.get(&previous_key).await?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        
        // 计算滑动窗口内的实际计数
        // 当前窗口的时间比例
        let window_progress = (now % window_seconds) as f64 / window_seconds as f64;
        let actual_count = (current_count as f64 + previous_count as f64 * (1.0 - window_progress)) as u64;
        
        let allowed = actual_count < limit;
        let remaining = if actual_count < limit {
            limit - actual_count
        } else {
            0
        };
        
        let reset_time = (current_window + 1) * window_seconds;
        
        Ok((allowed, remaining, reset_time))
    }

    async fn increment(&self, key: &str, window_seconds: u64) 
        -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let base_key = self.rate_limit_key(key);
        let current_window = now / window_seconds;
        let window_key = self.window_key(&base_key, current_window);
        
        // 增加计数
        let current_count: u64 = self.cache.get(&window_key).await?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        
        let new_count = current_count + 1;
        self.cache.set(&window_key, &new_count.to_string(), Some(window_seconds + 1)).await?;
        
        Ok(new_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 注意：这些测试需要Redis连接，在实际环境中运行
    // 这里只提供测试框架
}

