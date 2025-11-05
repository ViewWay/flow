use async_trait::async_trait;
use std::sync::Arc;
use redis::Client as RedisClient;

/// Cache trait 定义缓存操作
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;
    async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// RedisCache 使用Redis实现的缓存
pub struct RedisCache {
    client: Arc<RedisClient>,
}

impl RedisCache {
    pub fn new(client: Arc<RedisClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        if let Some(ttl) = ttl {
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl)
                .arg(value)
                .query_async(&mut conn)
                .await?;
        } else {
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .query_async(&mut conn)
                .await?;
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_trait_object_safety() {
        // 测试trait可以作为trait object使用
        fn takes_cache(_cache: &dyn Cache) {}
    }
}
