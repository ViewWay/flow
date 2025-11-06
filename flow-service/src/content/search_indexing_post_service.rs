use async_trait::async_trait;
use flow_api::extension::ListResult;
use flow_domain::content::Post;
use crate::content::{PostService, PostRequest, PostQuery, ListedPost, ContentWrapper};
use crate::search::{SearchService, DocumentConverter};
use std::sync::Arc;
use tracing::{debug, warn};

/// 带搜索索引的Post服务包装器
/// 在Post发布/更新/删除时自动更新搜索索引
pub struct SearchIndexingPostService {
    inner: Arc<dyn PostService>,
    search_service: Arc<dyn SearchService>,
}

impl SearchIndexingPostService {
    pub fn new(inner: Arc<dyn PostService>, search_service: Arc<dyn SearchService>) -> Self {
        Self { inner, search_service }
    }
    
    /// 更新Post的搜索索引
    async fn update_search_index(&self, post: &Post) {
        // 如果Post已删除，从索引中删除
        if post.is_deleted() {
            let doc_id = DocumentConverter::post_doc_id(&post.metadata.name);
            if let Err(e) = self.search_service.delete_document(vec![doc_id]).await {
                warn!("Failed to delete post from search index: {}", e);
            }
            return;
        }
        
        // 如果Post已发布且公开，更新索引
        if post.is_published() && post.is_public() {
            // 获取发布内容
            match self.inner.get_release_content(&post.metadata.name).await {
                Ok(content) => {
                    let halo_doc = DocumentConverter::convert_post(post, &content);
                    if let Err(e) = self.search_service.add_or_update(vec![halo_doc]).await {
                        warn!("Failed to update post in search index: {}", e);
                    } else {
                        debug!("Updated post {} in search index", post.metadata.name);
                    }
                }
                Err(e) => {
                    // 如果没有发布内容，尝试删除索引
                    debug!("Post {} has no release content, removing from index: {}", post.metadata.name, e);
                    let doc_id = DocumentConverter::post_doc_id(&post.metadata.name);
                    let _ = self.search_service.delete_document(vec![doc_id]).await;
                }
            }
        } else {
            // 如果未发布或未公开，从索引中删除
            let doc_id = DocumentConverter::post_doc_id(&post.metadata.name);
            if let Err(e) = self.search_service.delete_document(vec![doc_id]).await {
                warn!("Failed to delete post from search index: {}", e);
            }
        }
    }
}

#[async_trait]
impl PostService for SearchIndexingPostService {
    async fn list_post(&self, query: PostQuery) -> Result<ListResult<ListedPost>, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.list_post(query).await
    }
    
    async fn draft_post(&self, request: PostRequest) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        let post = self.inner.draft_post(request).await?;
        // 草稿不需要索引
        Ok(post)
    }
    
    async fn update_post(&self, request: PostRequest) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        let post = self.inner.update_post(request).await?;
        // 更新后检查是否需要更新索引
        self.update_search_index(&post).await;
        Ok(post)
    }
    
    async fn update_by(&self, post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        let updated_post = self.inner.update_by(post).await?;
        // 更新后检查是否需要更新索引
        self.update_search_index(&updated_post).await;
        Ok(updated_post)
    }
    
    async fn get_head_content(&self, post_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_head_content(post_name).await
    }
    
    async fn get_release_content(&self, post_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_release_content(post_name).await
    }
    
    async fn get_content(&self, snapshot_name: &str, base_snapshot_name: Option<&str>) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_content(snapshot_name, base_snapshot_name).await
    }
    
    async fn publish(&self, post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        let published_post = self.inner.publish(post).await?;
        // 发布后更新索引
        self.update_search_index(&published_post).await;
        Ok(published_post)
    }
    
    async fn unpublish(&self, post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        let unpublished_post = self.inner.unpublish(post).await?;
        // 取消发布后从索引中删除
        self.update_search_index(&unpublished_post).await;
        Ok(unpublished_post)
    }
    
    async fn get_by_username(&self, post_name: &str, username: &str) -> Result<Option<Post>, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_by_username(post_name, username).await
    }
    
    async fn revert_to_snapshot(&self, post_name: &str, snapshot_name: &str) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        let post = self.inner.revert_to_snapshot(post_name, snapshot_name).await?;
        // 恢复后更新索引
        self.update_search_index(&post).await;
        Ok(post)
    }
    
    async fn delete_content(&self, post_name: &str, snapshot_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.delete_content(post_name, snapshot_name).await
    }
    
    async fn recycle(&self, post_name: &str, username: &str) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        let post = self.inner.recycle(post_name, username).await?;
        // 回收后从索引中删除
        self.update_search_index(&post).await;
        Ok(post)
    }
}

