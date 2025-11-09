use flow_api::search::{SearchOption, SearchResult, SearchEngine, HaloDocument};
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub mod document_converter;
pub mod cached;

pub use document_converter::DocumentConverter;
pub use cached::{CachedSearchService, SearchStats};

/// 搜索服务trait
#[async_trait]
pub trait SearchService: Send + Sync {
    /// 执行搜索
    async fn search(&self, option: SearchOption) -> Result<SearchResult>;
    
    /// 添加或更新文档到搜索索引
    async fn add_or_update(&self, documents: Vec<HaloDocument>) -> Result<()>;
    
    /// 从搜索索引中删除文档
    async fn delete_document(&self, doc_ids: Vec<String>) -> Result<()>;
    
    /// 删除所有文档
    async fn delete_all(&self) -> Result<()>;
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
    
    async fn add_or_update(&self, documents: Vec<HaloDocument>) -> Result<()> {
        if !self.engine.available() {
            anyhow::bail!("Search engine is not available");
        }
        
        self.engine.add_or_update(documents).await
            .map_err(|e| anyhow::anyhow!("Failed to add or update documents: {}", e))
    }
    
    async fn delete_document(&self, doc_ids: Vec<String>) -> Result<()> {
        if !self.engine.available() {
            anyhow::bail!("Search engine is not available");
        }
        
        self.engine.delete_document(doc_ids).await
            .map_err(|e| anyhow::anyhow!("Failed to delete documents: {}", e))
    }
    
    async fn delete_all(&self) -> Result<()> {
        if !self.engine.available() {
            anyhow::bail!("Search engine is not available");
        }
        
        self.engine.delete_all().await
            .map_err(|e| anyhow::anyhow!("Failed to delete all documents: {}", e))
    }
}

