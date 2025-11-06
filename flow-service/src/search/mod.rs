use flow_api::search::{SearchOption, SearchResult, SearchEngine};
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

/// 搜索服务trait
#[async_trait]
pub trait SearchService: Send + Sync {
    /// 执行搜索
    async fn search(&self, option: SearchOption) -> Result<SearchResult>;
}

/// 默认搜索服务实现
pub struct DefaultSearchService {
    engine: Arc<dyn SearchEngine>,
}

impl DefaultSearchService {
    pub fn new(engine: Arc<dyn SearchEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl SearchService for DefaultSearchService {
    async fn search(&self, option: SearchOption) -> Result<SearchResult> {
        if !self.engine.available() {
            anyhow::bail!("Search engine is not available");
        }
        
        self.engine.search(option).await
            .map_err(|e| anyhow::anyhow!("Search failed: {}", e))
    }
}

