use flow_api::extension::{Extension, ListOptions};
use crate::index::IndicesManager;
use crate::index::query_visitor::QueryVisitor;
use std::sync::Arc;

/// IndexEngine trait 定义索引引擎的操作
pub trait IndexEngine: Send + Sync {
    /// 插入扩展对象到索引
    fn insert<E: Extension + 'static>(&self, extensions: &[E]);
    
    /// 更新索引中的扩展对象
    fn update<E: Extension + 'static>(&self, extensions: &[E]);
    
    /// 从索引中删除扩展对象
    fn delete<E: Extension + 'static>(&self, extensions: &[E]);
    
    /// 检索扩展对象主键列表
    fn retrieve<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 计数扩展对象
    fn count<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取索引管理器
    fn get_indices_manager(&self) -> &IndicesManager;
}

/// DefaultIndexEngine 默认索引引擎实现
pub struct DefaultIndexEngine {
    indices_manager: Arc<IndicesManager>,
}

impl DefaultIndexEngine {
    pub fn new() -> Self {
        Self {
            indices_manager: Arc::new(IndicesManager::new()),
        }
    }
    
    pub fn with_indices_manager(indices_manager: Arc<IndicesManager>) -> Self {
        Self { indices_manager }
    }
}

impl Default for DefaultIndexEngine {
    fn default() -> Self {
        Self::new()
    }
}

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
    
    fn retrieve<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let condition = options.to_condition();
        
        // 从IndicesManager获取Indices<E>
        let indices = self.indices_manager.get::<E>()
            .map_err(|e| format!("Failed to get indices: {}", e))?;
        
        let mut visitor = QueryVisitor::new(indices);
        visitor.visit(&condition)?;
        Ok(visitor.get_result().into_iter().collect())
    }
    
    fn count<E: Extension + 'static>(
        &self,
        options: &ListOptions,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.retrieve::<E>(options)?;
        Ok(result.len())
    }
    
    fn get_indices_manager(&self) -> &IndicesManager {
        &self.indices_manager
    }
}

