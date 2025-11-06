use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::content::Category;
use std::sync::Arc;

/// Category服务trait
#[async_trait]
pub trait CategoryService: Send + Sync {
    async fn create(&self, category: Category) -> Result<Category, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&self, category: Category) -> Result<Category, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get(&self, name: &str) -> Result<Option<Category>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<Category>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct DefaultCategoryService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultCategoryService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> CategoryService for DefaultCategoryService<C> {
    async fn create(&self, category: Category) -> Result<Category, Box<dyn std::error::Error + Send + Sync>> {
        self.client.create(category).await
    }

    async fn update(&self, category: Category) -> Result<Category, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(category).await
    }

    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete::<Category>(name).await
    }

    async fn get(&self, name: &str) -> Result<Option<Category>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.fetch(name).await
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<Category>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list(options).await
    }
}

