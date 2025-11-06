use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::content::SinglePage;
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
}

