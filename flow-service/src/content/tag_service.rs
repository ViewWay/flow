use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::content::Tag;
use std::sync::Arc;

/// Tag服务trait
#[async_trait]
pub trait TagService: Send + Sync {
    async fn create(&self, tag: Tag) -> Result<Tag, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&self, tag: Tag) -> Result<Tag, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get(&self, name: &str) -> Result<Option<Tag>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<Tag>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct DefaultTagService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultTagService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> TagService for DefaultTagService<C> {
    async fn create(&self, tag: Tag) -> Result<Tag, Box<dyn std::error::Error + Send + Sync>> {
        self.client.create(tag).await
    }

    async fn update(&self, tag: Tag) -> Result<Tag, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(tag).await
    }

    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete::<Tag>(name).await
    }

    async fn get(&self, name: &str) -> Result<Option<Tag>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.fetch(name).await
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<Tag>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list(options).await
    }
}

