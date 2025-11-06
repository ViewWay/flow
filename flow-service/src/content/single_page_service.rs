use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::content::{SinglePage, Snapshot};
use crate::content::{ContentWrapper, patch_utils};
use std::sync::Arc;

/// SinglePage服务trait
#[async_trait]
pub trait SinglePageService: Send + Sync {
    async fn create(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get(&self, name: &str) -> Result<Option<SinglePage>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<SinglePage>, Box<dyn std::error::Error + Send + Sync>>;
    async fn publish(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>>;
    async fn unpublish(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取头部内容（最新版本）
    async fn get_head_content(&self, page_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取发布内容
    async fn get_release_content(&self, page_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取指定快照的内容
    async fn get_content(&self, snapshot_name: &str, base_snapshot_name: Option<&str>) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct DefaultSinglePageService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultSinglePageService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> SinglePageService for DefaultSinglePageService<C> {
    async fn create(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        self.client.create(page).await
    }

    async fn update(&self, page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(page).await
    }

    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete::<SinglePage>(name).await
    }

    async fn get(&self, name: &str) -> Result<Option<SinglePage>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.fetch(name).await
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<SinglePage>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list(options).await
    }

    async fn publish(&self, mut page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        use flow_domain::content::constant;
        if page.metadata.labels.is_none() {
            page.metadata.labels = Some(std::collections::HashMap::new());
        }
        if let Some(ref mut labels) = page.metadata.labels {
            labels.insert(constant::POST_PUBLISHED_LABEL.to_string(), "true".to_string());
        }
        page.spec.publish = Some(true);
        self.client.update(page).await
    }

    async fn unpublish(&self, mut page: SinglePage) -> Result<SinglePage, Box<dyn std::error::Error + Send + Sync>> {
        use flow_domain::content::constant;
        if let Some(ref mut labels) = page.metadata.labels {
            labels.insert(constant::POST_PUBLISHED_LABEL.to_string(), "false".to_string());
        }
        page.spec.publish = Some(false);
        self.client.update(page).await
    }
    
    async fn get_head_content(&self, page_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        let page = self.client.fetch::<SinglePage>(page_name).await?
            .ok_or_else(|| "SinglePage not found")?;
        
        let head_snapshot = page.spec.head_snapshot.as_ref()
            .ok_or_else(|| "Head snapshot not found")?;
        let base_snapshot = page.spec.base_snapshot.as_ref()
            .ok_or_else(|| "Base snapshot not found")?;
        
        self.get_content(head_snapshot, Some(base_snapshot)).await
    }
    
    async fn get_release_content(&self, page_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        let page = self.client.fetch::<SinglePage>(page_name).await?
            .ok_or_else(|| "SinglePage not found")?;
        
        let release_snapshot = page.spec.release_snapshot.as_ref()
            .ok_or_else(|| "Release snapshot not found")?;
        let base_snapshot = page.spec.base_snapshot.as_ref()
            .ok_or_else(|| "Base snapshot not found")?;
        
        self.get_content(release_snapshot, Some(base_snapshot)).await
    }
    
    async fn get_content(&self, snapshot_name: &str, base_snapshot_name: Option<&str>) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        let base_snapshot_name = base_snapshot_name.ok_or_else(|| "Base snapshot name is required")?;
        
        // 获取base snapshot
        let base_snapshot = self.client.fetch::<Snapshot>(base_snapshot_name).await?
            .ok_or_else(|| "Base snapshot not found")?;
        
        // 检查是否是base snapshot
        if !base_snapshot.is_base_snapshot() {
            return Err("The snapshot is not a base snapshot".into());
        }
        
        // 如果snapshot_name等于base_snapshot_name，直接返回base snapshot的内容
        if snapshot_name == base_snapshot_name {
            let raw = base_snapshot.spec.raw_patch.as_deref().unwrap_or("");
            let content = base_snapshot.spec.content_patch.as_deref().unwrap_or("");
            return Ok(ContentWrapper {
                snapshot_name: base_snapshot.metadata.name.clone(),
                raw: raw.to_string(),
                content: content.to_string(),
                raw_type: base_snapshot.spec.raw_type.clone(),
            });
        }
        
        // 获取patch snapshot
        let patch_snapshot = self.client.fetch::<Snapshot>(snapshot_name).await?
            .ok_or_else(|| "Snapshot not found")?;
        
        // 应用patch
        let base_raw = base_snapshot.spec.raw_patch.as_deref().unwrap_or("");
        let base_content = base_snapshot.spec.content_patch.as_deref().unwrap_or("");
        
        let raw_patch = patch_snapshot.spec.raw_patch.as_deref().unwrap_or("");
        let content_patch = patch_snapshot.spec.content_patch.as_deref().unwrap_or("");
        
        let patched_raw = if raw_patch.is_empty() {
            base_raw.to_string()
        } else {
            patch_utils::apply_patch(base_raw, raw_patch)?
        };
        
        let patched_content = if content_patch.is_empty() {
            base_content.to_string()
        } else {
            patch_utils::apply_patch(base_content, content_patch)?
        };
        
        Ok(ContentWrapper {
            snapshot_name: patch_snapshot.metadata.name.clone(),
            raw: patched_raw,
            content: patched_content,
            raw_type: patch_snapshot.spec.raw_type.clone(),
        })
    }
}

