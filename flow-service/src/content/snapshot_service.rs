use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::content::Snapshot;
use std::sync::Arc;

/// Snapshot服务trait
#[async_trait]
pub trait SnapshotService: Send + Sync {
    async fn create(&self, snapshot: Snapshot) -> Result<Snapshot, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&self, snapshot: Snapshot) -> Result<Snapshot, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get(&self, name: &str) -> Result<Option<Snapshot>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<Snapshot>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_by_subject(&self, subject_ref: &flow_domain::content::SubjectRef) -> Result<Vec<Snapshot>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct DefaultSnapshotService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultSnapshotService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> SnapshotService for DefaultSnapshotService<C> {
    async fn create(&self, snapshot: Snapshot) -> Result<Snapshot, Box<dyn std::error::Error + Send + Sync>> {
        self.client.create(snapshot).await
    }

    async fn update(&self, snapshot: Snapshot) -> Result<Snapshot, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(snapshot).await
    }

    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete::<Snapshot>(name).await
    }

    async fn get(&self, name: &str) -> Result<Option<Snapshot>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.fetch(name).await
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<Snapshot>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list(options).await
    }

    async fn list_by_subject(&self, subject_ref: &flow_domain::content::SubjectRef) -> Result<Vec<Snapshot>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现根据主题查询快照
        let options = ListOptions::default();
        let result = self.list(options).await?;
        let snapshots: Vec<Snapshot> = result.items
            .into_iter()
            .filter(|s| {
                s.spec.subject_ref.group == subject_ref.group
                    && s.spec.subject_ref.kind == subject_ref.kind
                    && s.spec.subject_ref.name == subject_ref.name
            })
            .collect();
        Ok(snapshots)
    }
}

