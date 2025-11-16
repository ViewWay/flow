use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::attachment::{Group, GroupStatus};
use flow_infra::extension::ReactiveExtensionClient;
use crate::attachment::AttachmentService;
use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;

/// Group服务trait
#[async_trait]
pub trait GroupService: Send + Sync {
    async fn create(&self, group: Group) -> Result<Group>;
    async fn update(&self, group: Group) -> Result<Group>;
    async fn delete(&self, name: &str) -> Result<()>;
    async fn get(&self, name: &str) -> Result<Option<Group>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<Group>>;
    /// 更新分组的附件计数
    async fn update_attachment_count(&self, name: &str) -> Result<Group>;
}

/// 默认Group服务实现
pub struct DefaultGroupService {
    client: Arc<ReactiveExtensionClient>,
    attachment_service: Arc<dyn AttachmentService>,
}

impl DefaultGroupService {
    pub fn new(
        client: Arc<ReactiveExtensionClient>,
        attachment_service: Arc<dyn AttachmentService>,
    ) -> Self {
        Self {
            client,
            attachment_service,
        }
    }
}

#[async_trait]
impl GroupService for DefaultGroupService {
    async fn create(&self, group: Group) -> Result<Group> {
        let mut group_with_status = group;
        // 初始化状态
        group_with_status.status = Some(GroupStatus {
            update_timestamp: Some(Utc::now()),
            total_attachments: Some(0),
        });
        
        self.client.create(group_with_status).await
            .map_err(|e| anyhow::anyhow!("Failed to create group: {}", e))
    }

    async fn update(&self, mut group: Group) -> Result<Group> {
        // 更新时保留或更新状态
        if group.status.is_none() {
            group.status = Some(GroupStatus {
                update_timestamp: Some(Utc::now()),
                total_attachments: Some(0),
            });
        } else {
            // 更新时间戳
            if let Some(ref mut status) = group.status {
                status.update_timestamp = Some(Utc::now());
            }
        }
        
        self.client.update(group).await
            .map_err(|e| anyhow::anyhow!("Failed to update group: {}", e))
    }

    async fn delete(&self, name: &str) -> Result<()> {
        self.client.delete::<Group>(name).await
            .map_err(|e| anyhow::anyhow!("Failed to delete group: {}", e))
    }

    async fn get(&self, name: &str) -> Result<Option<Group>> {
        self.client.fetch(name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch group: {}", e))
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<Group>> {
        self.client.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list groups: {}", e))
    }

    async fn update_attachment_count(&self, name: &str) -> Result<Group> {
        // 查询该分组下的附件数量
        use flow_api::extension::query::Condition;
        let mut options = ListOptions::default();
        options.condition = Some(Condition::Equal {
            index_name: "spec.groupName".to_string(),
            value: serde_json::Value::String(name.to_string()),
        });
        
        let attachments = self.attachment_service.list(options).await?;
        let count = attachments.len() as u64;
        
        // 更新分组状态
        let mut group = self.get(name).await?
            .ok_or_else(|| anyhow::anyhow!("Group not found: {}", name))?;
        
        group.status = Some(GroupStatus {
            update_timestamp: Some(Utc::now()),
            total_attachments: Some(count),
        });
        
        self.update(group).await
    }
}

