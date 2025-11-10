use async_trait::async_trait;
use flow_domain::notification::{Reason, Subscription, SubscriptionSubscriber, InterestReason, SubscriptionSpec};
use crate::notification::{NotificationService, NotificationSender, NotificationCenter};
use flow_api::extension::{ExtensionClient, ListOptions, query::Condition};
use flow_infra::extension::ReactiveExtensionClient;
use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;

/// 默认通知中心实现
pub struct DefaultNotificationCenter {
    extension_client: Arc<ReactiveExtensionClient>,
    notification_service: Arc<dyn NotificationService>,
    sender: Arc<dyn NotificationSender>,
}

impl DefaultNotificationCenter {
    pub fn new(
        extension_client: Arc<ReactiveExtensionClient>,
        notification_service: Arc<dyn NotificationService>,
        sender: Arc<dyn NotificationSender>,
    ) -> Self {
        Self {
            extension_client,
            notification_service,
            sender,
        }
    }
}

#[async_trait]
impl NotificationCenter for DefaultNotificationCenter {
    async fn notify(&self, reason: Reason) -> Result<()> {
        // TODO: 实现通知逻辑
        // 1. 查找所有订阅该reason的Subscription
        // 2. 为每个订阅者创建Notification
        // 3. 使用NotificationSender发送通知
        
        // 临时实现：直接创建站内通知
        use flow_domain::notification::{Notification, NotificationSpec};
        use flow_api::extension::Metadata;
        
        // 这里简化实现，实际应该查找订阅者
        // 假设reason.spec.subject包含接收者信息
        let notification = Notification {
            metadata: Metadata::new(Uuid::new_v4().to_string()),
            spec: NotificationSpec {
                recipient: reason.spec.author.clone(),
                reason: reason.metadata.name.clone(),
                title: format!("Notification: {}", reason.spec.subject.title),
                raw_content: format!("Reason: {}", reason.spec.reason_type),
                html_content: format!("<p>Reason: {}</p>", reason.spec.reason_type),
                unread: Some(true),
                last_read_at: None,
            },
        };
        
        self.notification_service.create(notification).await?;
        
        Ok(())
    }

    async fn subscribe(
        &self,
        subscriber: SubscriptionSubscriber,
        interest_reason: InterestReason,
    ) -> Result<Subscription> {
        use flow_domain::notification::{Subscription, SubscriptionSpec};
        use flow_api::extension::Metadata;
        
        let subscription = Subscription {
            metadata: Metadata::new(Uuid::new_v4().to_string()),
            spec: SubscriptionSpec {
                subscriber: subscriber.clone(),
                unsubscribe_token: Subscription::generate_unsubscribe_token(),
                reason: interest_reason,
                disabled: Some(false),
            },
        };
        
        self.extension_client.create(subscription.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to create subscription: {}", e))?;
        
        Ok(subscription)
    }

    async fn unsubscribe(&self, subscriber: &SubscriptionSubscriber) -> Result<()> {
        // 查找该订阅者的所有订阅
        let mut options = ListOptions::default();
        options.condition = Some(Condition::Equal {
            index_name: "spec.subscriber.name".to_string(),
            value: serde_json::Value::String(subscriber.name.clone()),
        });
        
        let result = self.extension_client.list::<Subscription>(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list subscriptions: {}", e))?;
        
        // 删除所有订阅
        for subscription in result.items {
            self.extension_client.delete::<Subscription>(&subscription.metadata.name).await
                .map_err(|e| anyhow::anyhow!("Failed to delete subscription: {}", e))?;
        }
        
        Ok(())
    }

    async fn unsubscribe_reason(
        &self,
        subscriber: &SubscriptionSubscriber,
        interest_reason: &InterestReason,
    ) -> Result<()> {
        // 查找匹配的订阅
        let mut options = ListOptions::default();
        // TODO: 实现更复杂的查询条件匹配
        options.condition = Some(Condition::Equal {
            index_name: "spec.subscriber.name".to_string(),
            value: serde_json::Value::String(subscriber.name.clone()),
        });
        
        let result = self.extension_client.list::<Subscription>(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list subscriptions: {}", e))?;
        
        // 删除匹配的订阅
        for subscription in result.items {
            if subscription.spec.reason.reason_type == interest_reason.reason_type {
                self.extension_client.delete::<Subscription>(&subscription.metadata.name).await
                    .map_err(|e| anyhow::anyhow!("Failed to delete subscription: {}", e))?;
            }
        }
        
        Ok(())
    }
}

