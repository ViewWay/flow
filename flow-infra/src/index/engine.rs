use flow_api::extension::{Extension, ListOptions};
use flow_api::search::SearchEngine;
use crate::index::IndicesManager;
use crate::index::query_visitor::QueryVisitor;
use crate::index::fulltext_field_mapping::FulltextFieldMapping;
use crate::index::doc_type_converter::DocTypeProvider;
use std::sync::Arc;
use async_trait::async_trait;

/// IndexEngine trait 定义索引引擎的操作
#[async_trait]
pub trait IndexEngine: Send + Sync {
    /// 插入扩展对象到索引
    fn insert<E: Extension + 'static>(&self, extensions: &[E]);
    
    /// 更新索引中的扩展对象
    fn update<E: Extension + 'static>(&self, extensions: &[E]);
    
    /// 从索引中删除扩展对象
    fn delete<E: Extension + 'static>(&self, extensions: &[E]);
    
    /// 检索扩展对象主键列表
    async fn retrieve<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 计数扩展对象
    async fn count<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取索引管理器
    fn get_indices_manager(&self) -> &IndicesManager;
}

/// DefaultIndexEngine 默认索引引擎实现
pub struct DefaultIndexEngine {
    indices_manager: Arc<IndicesManager>,
    search_engine: Option<Arc<dyn SearchEngine>>,
    fulltext_mapping: Arc<FulltextFieldMapping>,
}

impl DefaultIndexEngine {
    pub fn new() -> Self {
        Self {
            indices_manager: Arc::new(IndicesManager::new()),
            search_engine: None,
            fulltext_mapping: Arc::new(FulltextFieldMapping::default()),
        }
    }
    
    pub fn with_indices_manager(indices_manager: Arc<IndicesManager>) -> Self {
        Self {
            indices_manager,
            search_engine: None,
            fulltext_mapping: Arc::new(FulltextFieldMapping::default()),
        }
    }
    
    pub fn with_search_engine(
        indices_manager: Arc<IndicesManager>,
        search_engine: Option<Arc<dyn SearchEngine>>,
        fulltext_mapping: Arc<FulltextFieldMapping>,
    ) -> Self {
        Self {
            indices_manager,
            search_engine,
            fulltext_mapping,
        }
    }
}

impl Default for DefaultIndexEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IndexEngine for DefaultIndexEngine {
    fn insert<E: Extension + 'static>(&self, extensions: &[E]) {
        if let Ok(indices) = self.indices_manager.get::<E>() {
            for extension in extensions {
                indices.insert(extension);
            }
        }
    }
    
    fn update<E: Extension + 'static>(&self, extensions: &[E]) {
        if let Ok(indices) = self.indices_manager.get::<E>() {
            for extension in extensions {
                indices.update(extension);
            }
        }
    }
    
    fn delete<E: Extension + 'static>(&self, extensions: &[E]) {
        if let Ok(indices) = self.indices_manager.get::<E>() {
            for extension in extensions {
                indices.delete(extension);
            }
        }
    }
    
    async fn retrieve<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let condition = options.to_condition();
        
        // 从IndicesManager获取Indices<E>
        let indices = self.indices_manager.get::<E>()
            .map_err(|e| format!("Failed to get indices: {}", e))?;
        
        // 尝试获取文档类型（如果类型实现了 DocTypeProvider）
        // 由于 Rust 的类型系统限制，我们无法在运行时检查 trait 实现
        // 所以这里总是返回 None，让 QueryVisitor 使用字符串匹配
        // 如果需要全文搜索，需要在调用时确保类型实现了 DocTypeProvider
        // 并在 IndexEngine 层面提供特殊实现
        let mut visitor = QueryVisitor::new(
            indices,
            self.search_engine.as_ref().map(|e| Arc::clone(e)),
            Arc::clone(&self.fulltext_mapping),
        );
        visitor.visit(&condition).await?;
        Ok(visitor.get_result().into_iter().collect())
    }
    
    async fn count<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.retrieve::<E>(options).await?;
        Ok(result.len())
    }
    
    fn get_indices_manager(&self) -> &IndicesManager {
        &self.indices_manager
    }
}

// 辅助实现：为实现了 DocTypeProvider 的类型提供全文搜索支持
impl DefaultIndexEngine {
    /// 检索扩展对象主键列表（支持全文搜索的版本）
    /// 仅当类型实现了 DocTypeProvider 时可用
    pub async fn retrieve_with_fulltext<E: Extension + DocTypeProvider + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let condition = options.to_condition();
        
        // 从IndicesManager获取Indices<E>
        let indices = self.indices_manager.get::<E>()
            .map_err(|e| format!("Failed to get indices: {}", e))?;
        
        // 获取文档类型
        let doc_type = <E as DocTypeProvider>::doc_type();
        
        let mut visitor = QueryVisitor::with_doc_type(
            indices,
            self.search_engine.as_ref().map(|e| Arc::clone(e)),
            Arc::clone(&self.fulltext_mapping),
            doc_type,
        );
        visitor.visit(&condition).await?;
        Ok(visitor.get_result().into_iter().collect())
    }
}

