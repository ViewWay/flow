use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::notification::{Notification, NotificationSpec};
use flow_infra::extension::ReactiveExtensionClient;
use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;

/// Notification服务trait
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// 创建通知
    async fn create(&self, notification: Notification) -> Result<Notification>;
    
    /// 更新通知
    async fn update(&self, notification: Notification) -> Result<Notification>;
    
    /// 删除通知
    async fn delete(&self, name: &str) -> Result<()>;
    
    /// 获取通知
    async fn get(&self, name: &str) -> Result<Option<Notification>>;
    
    /// 列出通知
    async fn list(&self, options: ListOptions) -> Result<ListResult<Notification>>;
    
    /// 标记通知为已读
    async fn mark_as_read(&self, name: &str) -> Result<Notification>;
    
    /// 标记所有通知为已读（针对特定用户）
    async fn mark_all_as_read(&self, recipient: &str) -> Result<()>;
    
    /// 获取未读通知数量（针对特定用户）
    async fn get_unread_count(&self, recipient: &str) -> Result<u64>;
}

/// 默认Notification服务实现
pub struct DefaultNotificationService {
    client: Arc<ReactiveExtensionClient>,
}

impl DefaultNotificationService {
    pub fn new(client: Arc<ReactiveExtensionClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl NotificationService for DefaultNotificationService {
    async fn create(&self, notification: Notification) -> Result<Notification> {
        self.client.create(notification).await
            .map_err(|e| anyhow::anyhow!("Failed to create notification: {}", e))
    }

    async fn update(&self, notification: Notification) -> Result<Notification> {
        self.client.update(notification).await
            .map_err(|e| anyhow::anyhow!("Failed to update notification: {}", e))
    }

    async fn delete(&self, name: &str) -> Result<()> {
        self.client.delete::<Notification>(name).await
            .map_err(|e| anyhow::anyhow!("Failed to delete notification: {}", e))
    }

    async fn get(&self, name: &str) -> Result<Option<Notification>> {
        self.client.fetch(name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch notification: {}", e))
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<Notification>> {
        self.client.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list notifications: {}", e))
    }

    async fn mark_as_read(&self, name: &str) -> Result<Notification> {
        let mut notification = self.get(name).await?
            .ok_or_else(|| anyhow::anyhow!("Notification not found: {}", name))?;
        
        // 更新为已读
        notification.spec.unread = Some(false);
        notification.spec.last_read_at = Some(Utc::now());
        
        self.update(notification).await
    }

    async fn mark_all_as_read(&self, recipient: &str) -> Result<()> {
        use flow_api::extension::query::Condition;
        let mut options = ListOptions::default();
        options.condition = Some(Condition::Equal {
            index_name: "spec.recipient".to_string(),
            value: serde_json::Value::String(recipient.to_string()),
        });
        
        let result = self.list(options).await?;
        let now = Utc::now();
        
        for mut notification in result.items {
            if notification.spec.unread.unwrap_or(true) {
                notification.spec.unread = Some(false);
                notification.spec.last_read_at = Some(now);
                let _ = self.update(notification).await;
            }
        }
        
        Ok(())
    }

    async fn get_unread_count(&self, recipient: &str) -> Result<u64> {
        use flow_api::extension::query::Condition;
        let mut options = ListOptions::default();
        options.condition = Some(Condition::Equal {
            index_name: "spec.recipient".to_string(),
            value: serde_json::Value::String(recipient.to_string()),
        });
        
        let result = self.list(options).await?;
        let count = result.items.iter()
            .filter(|n| n.spec.unread.unwrap_or(true))
            .count() as u64;
        
        Ok(count)
    }
}

