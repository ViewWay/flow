use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::attachment::PolicyTemplate;
use flow_infra::extension::ReactiveExtensionClient;
use std::sync::Arc;
use anyhow::Result;

/// PolicyTemplate服务trait
#[async_trait]
pub trait PolicyTemplateService: Send + Sync {
    async fn create(&self, template: PolicyTemplate) -> Result<PolicyTemplate>;
    async fn update(&self, template: PolicyTemplate) -> Result<PolicyTemplate>;
    async fn delete(&self, name: &str) -> Result<()>;
    async fn get(&self, name: &str) -> Result<Option<PolicyTemplate>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<PolicyTemplate>>;
}

/// 默认PolicyTemplate服务实现
pub struct DefaultPolicyTemplateService {
    client: Arc<ReactiveExtensionClient>,
}

impl DefaultPolicyTemplateService {
    pub fn new(client: Arc<ReactiveExtensionClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl PolicyTemplateService for DefaultPolicyTemplateService {
    async fn create(&self, template: PolicyTemplate) -> Result<PolicyTemplate> {
        self.client.create(template).await
            .map_err(|e| anyhow::anyhow!("Failed to create policy template: {}", e))
    }

    async fn update(&self, template: PolicyTemplate) -> Result<PolicyTemplate> {
        self.client.update(template).await
            .map_err(|e| anyhow::anyhow!("Failed to update policy template: {}", e))
    }

    async fn delete(&self, name: &str) -> Result<()> {
        self.client.delete::<PolicyTemplate>(name).await
            .map_err(|e| anyhow::anyhow!("Failed to delete policy template: {}", e))
    }

    async fn get(&self, name: &str) -> Result<Option<PolicyTemplate>> {
        self.client.fetch(name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch policy template: {}", e))
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<PolicyTemplate>> {
        self.client.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list policy templates: {}", e))
    }
}

