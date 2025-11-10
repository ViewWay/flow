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
    
    /// 查找匹配的订阅
    async fn find_matching_subscriptions(&self, reason: &Reason) -> Result<Vec<Subscription>> {
        let mut options = ListOptions::default();
        // 查找所有订阅（后续可以优化为只查询匹配reason_type的订阅）
        let result = self.extension_client.list::<Subscription>(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list subscriptions: {}", e))?;
        
        // 过滤匹配的订阅
        let matching: Vec<Subscription> = result.items.into_iter()
            .filter(|sub| self.matches_reason(sub, reason))
            .collect();
        
        Ok(matching)
    }
    
    /// 检查订阅是否匹配reason
    fn matches_reason(&self, subscription: &Subscription, reason: &Reason) -> bool {
        let interest_reason = &subscription.spec.reason;
        
        // 检查reason_type是否匹配
        if interest_reason.reason_type != reason.spec.reason_type {
            return false;
        }
        
        // 检查subject是否匹配
        if let Some(subject) = &interest_reason.subject {
            let reason_subject = &reason.spec.subject;
            
            // 检查apiVersion和kind
            if subject.api_version != reason_subject.api_version 
                || subject.kind != reason_subject.kind {
                return false;
            }
            
            // 如果subject指定了name，必须完全匹配
            if let Some(name) = &subject.name {
                if name != &reason_subject.name {
                    return false;
                }
            }
        }
        
        // TODO: 实现expression匹配逻辑（SpEL表达式）
        // 目前暂时忽略expression字段
        
        true
    }
}

#[async_trait]
impl NotificationCenter for DefaultNotificationCenter {
    async fn notify(&self, reason: Reason) -> Result<()> {
        // 1. 查找所有订阅该reason的Subscription
        let subscriptions = self.find_matching_subscriptions(&reason).await?;
        
        // 2. 为每个订阅者创建Notification
        for subscription in subscriptions {
            // 跳过禁用的订阅
            if subscription.spec.disabled.unwrap_or(false) {
                continue;
            }
            
            let subscriber_name = &subscription.spec.subscriber.name;
            
            // 创建站内通知
            use flow_domain::notification::{Notification, NotificationSpec};
            use flow_api::extension::Metadata;
            
            let notification = Notification {
                metadata: Metadata::new(Uuid::new_v4().to_string()),
                spec: NotificationSpec {
                    recipient: subscriber_name.clone(),
                    reason: reason.metadata.name.clone(),
                    title: format!("Notification: {}", reason.spec.subject.title),
                    raw_content: format!("Reason: {}", reason.spec.reason_type),
                    html_content: format!("<p>Reason: {}</p>", reason.spec.reason_type),
                    unread: Some(true),
                    last_read_at: None,
                },
            };
            
            // 创建通知（忽略错误，继续处理其他订阅者）
            if let Err(e) = self.notification_service.create(notification).await {
                tracing::warn!("Failed to create notification for subscriber {}: {}", subscriber_name, e);
            }
            
            // TODO: 使用NotificationSender发送其他类型的通知（邮件、短信等）
            // 这里可以扩展支持多种通知方式
        }
        
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

