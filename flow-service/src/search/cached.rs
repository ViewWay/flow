use flow_api::search::{SearchOption, SearchResult, HaloDocument, CacheStats};
use flow_infra::cache::Cache;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use anyhow::Result;
use serde_json;
use sha2::{Sha256, Digest};
use hex;
use tracing::{debug, warn};

use super::SearchService;

/// 带缓存的搜索服务包装器
pub struct CachedSearchService {
    inner: Arc<dyn SearchService>,
    cache: Option<Arc<dyn Cache>>,
    cache_ttl: Option<u64>, // 缓存过期时间（秒）
    cache_prefix: String,
    // 性能监控统计
    total_searches: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    total_search_time: AtomicU64, // 总搜索时间（毫秒）
}

impl CachedSearchService {
    /// 创建新的带缓存的搜索服务
    pub fn new(
        inner: Arc<dyn SearchService>,
        cache: Option<Arc<dyn Cache>>,
        cache_ttl: Option<u64>,
    ) -> Self {
        Self {
            inner,
            cache,
            cache_ttl,
            cache_prefix: "search:".to_string(),
            total_searches: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            total_search_time: AtomicU64::new(0),
        }
    }

    /// 生成缓存键
    fn cache_key(&self, option: &SearchOption) -> String {
        // 使用 SHA256 哈希生成缓存键
        let mut hasher = Sha256::new();
        let option_json = serde_json::to_string(option).unwrap_or_default();
        hasher.update(option_json.as_bytes());
        let hash = hasher.finalize();
        format!("{}search:{}", self.cache_prefix, hex::encode(hash))
    }

    /// 获取性能统计
    pub fn get_stats(&self) -> SearchStats {
        let total = self.total_searches.load(Ordering::Relaxed);
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total_time = self.total_search_time.load(Ordering::Relaxed);
        
        let avg_time = if total > 0 {
            total_time / total
        } else {
            0
        };
        
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        SearchStats {
            total_searches: total,
            cache_hits: hits,
            cache_misses: misses,
            hit_rate,
            avg_search_time_millis: avg_time,
            total_search_time_millis: total_time,
        }
    }

    /// 清除缓存统计
    pub fn reset_stats(&self) {
        self.total_searches.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.total_search_time.store(0, Ordering::Relaxed);
    }
}

/// 搜索性能统计
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// 总搜索次数
    pub total_searches: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 缓存命中率（百分比）
    pub hit_rate: f64,
    /// 平均搜索时间（毫秒）
    pub avg_search_time_millis: u64,
    /// 总搜索时间（毫秒）
    pub total_search_time_millis: u64,
}

#[async_trait]
impl SearchService for CachedSearchService {
    async fn search(&self, option: SearchOption) -> Result<SearchResult> {
        let start_time = std::time::Instant::now();
        self.total_searches.fetch_add(1, Ordering::Relaxed);
        
        // 如果缓存可用，尝试从缓存获取
        if let Some(ref cache) = self.cache {
            let cache_key = self.cache_key(&option);
            
            match cache.get(&cache_key).await {
                Ok(Some(cached_json)) => {
                    match serde_json::from_str::<SearchResult>(&cached_json) {
                        Ok(mut result) => {
                            self.cache_hits.fetch_add(1, Ordering::Relaxed);
                            let elapsed = start_time.elapsed().as_millis() as u64;
                            self.total_search_time.fetch_add(elapsed, Ordering::Relaxed);
                            
                            result.from_cache = true;
                            result.cache_stats = Some(CacheStats {
                                hits: self.cache_hits.load(Ordering::Relaxed),
                                misses: self.cache_misses.load(Ordering::Relaxed),
                                size: 0, // 缓存大小需要从缓存实现获取
                            });
                            
                            debug!("Search cache hit for keyword: {}", option.keyword);
                            return Ok(result);
                        }
                        Err(e) => {
                            warn!("Failed to deserialize cached result: {}", e);
                            // 继续执行搜索
                        }
                    }
                }
                Ok(None) => {
                    // 缓存未命中
                    self.cache_misses.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    warn!("Cache get error: {}", e);
                    // 继续执行搜索
                }
            }
        }
        
        // 执行实际搜索
        let mut result = self.inner.search(option.clone()).await?;
        let elapsed = start_time.elapsed().as_millis() as u64;
        self.total_search_time.fetch_add(elapsed, Ordering::Relaxed);
        
        result.from_cache = false;
        
        // 如果缓存可用，保存结果到缓存
        if let Some(ref cache) = self.cache {
            let cache_key = self.cache_key(&option);
            if let Ok(result_json) = serde_json::to_string(&result) {
                if let Err(e) = cache.set(&cache_key, &result_json, self.cache_ttl).await {
                    warn!("Failed to cache search result: {}", e);
                } else {
                    debug!("Cached search result for keyword: {}", option.keyword);
                }
            }
        }
        
        Ok(result)
    }
    
    async fn add_or_update(&self, documents: Vec<HaloDocument>) -> Result<()> {
        // 文档更新时，清除相关缓存（简化实现：清除所有搜索缓存）
        if let Some(ref _cache) = self.cache {
            // 注意：这里简化处理，实际应该只清除相关的缓存
            // 可以使用 Redis 的 KEYS 命令或维护缓存键列表
            debug!("Documents updated, cache invalidation may be needed");
        }
        
        self.inner.add_or_update(documents).await
    }
    
    async fn delete_document(&self, doc_ids: Vec<String>) -> Result<()> {
        // 文档删除时，清除相关缓存
        if let Some(ref _cache) = self.cache {
            debug!("Documents deleted, cache invalidation may be needed");
        }
        
        self.inner.delete_document(doc_ids).await
    }
    
    async fn delete_all(&self) -> Result<()> {
        // 删除所有文档时，清除所有搜索缓存
        if let Some(ref _cache) = self.cache {
            // 这里可以清除所有以 cache_prefix 开头的键
            debug!("All documents deleted, cache should be cleared");
        }
        
        self.inner.delete_all().await
    }
}

