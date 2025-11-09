use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::attachment::{Policy, PolicySpec};
use flow_infra::extension::ReactiveExtensionClient;
use std::sync::Arc;
use anyhow::Result;

/// Policy服务trait
#[async_trait]
pub trait PolicyService: Send + Sync {
    async fn create(&self, policy: Policy) -> Result<Policy>;
    async fn update(&self, policy: Policy) -> Result<Policy>;
    async fn delete(&self, name: &str) -> Result<()>;
    async fn get(&self, name: &str) -> Result<Option<Policy>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<Policy>>;
}

/// 默认Policy服务实现
pub struct DefaultPolicyService {
    client: Arc<ReactiveExtensionClient>,
}

impl DefaultPolicyService {
    pub fn new(client: Arc<ReactiveExtensionClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl PolicyService for DefaultPolicyService {
    async fn create(&self, policy: Policy) -> Result<Policy> {
        self.client.create(policy).await
            .map_err(|e| anyhow::anyhow!("Failed to create policy: {}", e))
    }

    async fn update(&self, policy: Policy) -> Result<Policy> {
        self.client.update(policy).await
            .map_err(|e| anyhow::anyhow!("Failed to update policy: {}", e))
    }

    async fn delete(&self, name: &str) -> Result<()> {
        self.client.delete::<Policy>(name).await
            .map_err(|e| anyhow::anyhow!("Failed to delete policy: {}", e))
    }

    async fn get(&self, name: &str) -> Result<Option<Policy>> {
        self.client.fetch(name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch policy: {}", e))
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<Policy>> {
        self.client.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list policies: {}", e))
    }
}

