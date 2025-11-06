use async_trait::async_trait;
use flow_api::extension::{ListOptions, ListResult};
use flow_domain::content::SinglePage;
use crate::content::{SinglePageService, ContentWrapper};
use crate::search::{SearchService, DocumentConverter};
use std::sync::Arc;
use tracing::{debug, warn};

/// 带搜索索引的SinglePage服务包装器
/// 在SinglePage发布/更新/删除时自动更新搜索索引
pub struct SearchIndexingSinglePageService {
    inner: Arc<dyn SinglePageService>,
    search_service: Arc<dyn SearchService>,
    // 注意：SinglePage的内容获取逻辑需要单独实现，暂时不依赖PostService
}

impl SearchIndexingSinglePageService {
    pub fn new(
        inner: Arc<dyn SinglePageService>,
        search_service: Arc<dyn SearchService>,
    ) -> Self {
        Self { inner, search_service }
    }
    
    /// 更新SinglePage的搜索索引
    async fn update_search_index(&self, page: &SinglePage) {
        // 如果SinglePage已删除，从索引中删除
        if page.spec.deleted.unwrap_or(false) {
            let doc_id = DocumentConverter::single_page_doc_id(&page.metadata.name);
            if let Err(e) = self.search_service.delete_document(vec![doc_id]).await {
                warn!("Failed to delete single page from search index: {}", e);
            }
            return;
        }
        
        // 如果SinglePage已发布且公开，更新索引
        let is_public = matches!(page.spec.visible, Some(flow_domain::content::VisibleEnum::Public) | None);
        if page.is_published() && is_public {
            // 获取发布内容（SinglePage使用与Post相同的内容获取方式）
            // 注意：这里需要根据SinglePage的实际实现来获取内容
            // 暂时使用get_release_content，但SinglePage可能需要不同的方法
            match self.get_release_content_internal(page).await {
                Ok(content) => {
                    let halo_doc = DocumentConverter::convert_single_page(page, &content);
                    if let Err(e) = self.search_service.add_or_update(vec![halo_doc]).await {
                        warn!("Failed to update single page in search index: {}", e);
                    } else {
                        debug!("Updated single page {} in search index", page.metadata.name);
                    }
                }
                Err(e) => {
                    // 如果没有发布内容，尝试删除索引
                    debug!("SinglePage {} has no release content, removing from index: {}", page.metadata.name, e);
                    let doc_id = DocumentConverter::single_page_doc_id(&page.metadata.name);
                    let _ = self.search_service.delete_document(vec![doc_id]).await;
                }
            }
        } else {
            // 如果未发布或未公开，从索引中删除
            let doc_id = DocumentConverter::single_page_doc_id(&page.metadata.name);
            if let Err(e) = self.search_service.delete_document(vec![doc_id]).await {
                warn!("Failed to delete single page from search index: {}", e);
            }
        }
    }
    
    /// 内部方法：获取SinglePage的发布内容
    async fn get_release_content_internal(&self, page: &SinglePage) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        // 使用SinglePageService的get_release_content方法获取内容
        self.inner.get_release_content(&page.metadata.name).await
    }
}

#[async_trait]
impl SinglePageService for SearchIndexingSinglePageService {
    async fn create(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        let created_page = self.inner.create(page).await?;
        // 创建后检查是否需要更新索引
        self.update_search_index(&created_page).await;
        Ok(created_page)
    }
    
    async fn update(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        let updated_page = self.inner.update(page).await?;
        // 更新后检查是否需要更新索引
        self.update_search_index(&updated_page).await;
        Ok(updated_page)
    }
    
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 删除前先获取页面信息，以便从索引中删除
        if let Ok(Some(page)) = self.inner.get(name).await {
            let doc_id = DocumentConverter::single_page_doc_id(name);
            let _ = self.search_service.delete_document(vec![doc_id]).await;
        }
        self.inner.delete(name).await
    }
    
    async fn get(&self, name: &str) -> Result<Option<SinglePage>, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get(name).await
    }
    
    async fn list(&self, options: ListOptions) -> Result<ListResult<SinglePage>, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.list(options).await
    }
    
    async fn publish(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        let published_page = self.inner.publish(page).await?;
        // 发布后更新索引
        self.update_search_index(&published_page).await;
        Ok(published_page)
    }
    
    async fn unpublish(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        let unpublished_page = self.inner.unpublish(page).await?;
        // 取消发布后从索引中删除
        self.update_search_index(&unpublished_page).await;
        Ok(unpublished_page)
    }
    
    async fn get_head_content(&self, page_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_head_content(page_name).await
    }
    
    async fn get_release_content(&self, page_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_release_content(page_name).await
    }
    
    async fn get_content(&self, snapshot_name: &str, base_snapshot_name: Option<&str>) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_content(snapshot_name, base_snapshot_name).await
    }
}

