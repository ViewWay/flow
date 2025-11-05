use flow_api::extension::{Extension, ExtensionClient, ListOptions, ListResult};
use crate::database::ExtensionRepository;
use crate::extension::converter::{ExtensionConverter, JSONExtensionConverter};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// ReactiveExtensionClient 响应式扩展客户端实现
pub struct ReactiveExtensionClient {
    repository: Arc<dyn ExtensionRepository>,
    converter: JSONExtensionConverter,
}

impl ReactiveExtensionClient {
    pub fn new(repository: Arc<dyn ExtensionRepository>) -> Self {
        Self {
            repository,
            converter: JSONExtensionConverter,
        }
    }
}

#[async_trait]
impl ExtensionClient for ReactiveExtensionClient {
    async fn create<E: Extension + Serialize>(&self, extension: E) -> Result<E, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.converter.convert_to(&extension)?;
        self.repository.save(store).await?;
        Ok(extension)
    }

    async fn update<E: Extension + Serialize>(&self, extension: E) -> Result<E, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现乐观锁检查
        let store = self.converter.convert_to(&extension)?;
        self.repository.save(store).await?;
        Ok(extension)
    }

    async fn delete<E: Extension>(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 根据GVK构建完整的存储名称
        self.repository.delete(name).await?;
        Ok(())
    }

    async fn fetch<E: Extension + for<'de> Deserialize<'de>>(&self, name: &str) -> Result<Option<E>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 根据GVK构建完整的存储名称
        if let Some(store) = self.repository.find_by_name(name).await? {
            let extension: E = self.converter.convert_from(&store)?;
            Ok(Some(extension))
        } else {
            Ok(None)
        }
    }

    async fn list<E: Extension + for<'de> Deserialize<'de>>(&self, options: ListOptions) -> Result<ListResult<E>, Box<dyn std::error::Error + Send + Sync>> {
        let stores = self.repository.list(options.clone()).await?;
        let items: Result<Vec<E>, _> = stores
            .iter()
            .map(|store| self.converter.convert_from(store))
            .collect();

        let items = items?;
        let total = items.len() as u64;
        let page = options.page.unwrap_or(0);
        let size = options.size.unwrap_or(10);

        Ok(ListResult::new(items, total, page, size))
    }
}

