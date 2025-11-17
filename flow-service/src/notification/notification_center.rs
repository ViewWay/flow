use async_trait::async_trait;
use flow_domain::notification::{Reason, Subscription, SubscriptionSubscriber, InterestReason};
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
    #[allow(dead_code)] // 保留用于未来扩展（邮件、短信等通知方式）
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
        let options = ListOptions::default();
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
            
            // 如果subject存在，以subject匹配结果为准（根据文档说明）
            return true;
        }
        
        // 如果没有subject，检查expression
        if let Some(expression) = &interest_reason.expression {
            return self.evaluate_expression(expression, reason);
        }
        
        // 既没有subject也没有expression，只匹配reason_type
        true
    }
    
    /// 评估SpEL表达式
    /// 表达式上下文包含：props（reason attributes）、subject、author
    /// 注意：evalexpr不支持嵌套的HashMap，所以我们使用点号访问路径，如props.owner
    fn evaluate_expression(&self, expression: &str, reason: &Reason) -> bool {
        use evalexpr::{eval_boolean_with_context, Value, HashMapContext, ContextWithMutableVariables};
        
        // 创建表达式上下文
        let mut context = HashMapContext::new();
        
        // 添加props（reason attributes）- 使用点号访问路径
        if let Some(attributes) = &reason.spec.attributes {
            for (key, value) in attributes {
                let var_name = format!("props.{}", key);
                context.set_value(var_name, Value::String(value.clone()))
                    .unwrap_or_else(|_| {
                        tracing::warn!("Failed to set props.{} in expression context", key);
                    });
            }
        }
        
        // 添加subject属性 - 使用点号访问路径
        context.set_value("subject.apiVersion".to_string(), Value::String(reason.spec.subject.api_version.clone()))
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to set subject.apiVersion in expression context");
            });
        context.set_value("subject.kind".to_string(), Value::String(reason.spec.subject.kind.clone()))
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to set subject.kind in expression context");
            });
        context.set_value("subject.name".to_string(), Value::String(reason.spec.subject.name.clone()))
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to set subject.name in expression context");
            });
        context.set_value("subject.title".to_string(), Value::String(reason.spec.subject.title.clone()))
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to set subject.title in expression context");
            });
        if let Some(url) = &reason.spec.subject.url {
            context.set_value("subject.url".to_string(), Value::String(url.clone()))
                .unwrap_or_else(|_| {
                    tracing::warn!("Failed to set subject.url in expression context");
                });
        }
        
        // 添加author
        context.set_value("author".to_string(), Value::String(reason.spec.author.clone()))
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to set author in expression context");
            });
        
        // 评估表达式
        // 注意：evalexpr使用点号访问嵌套属性，如props.owner == "guqing"
        match eval_boolean_with_context(expression, &context) {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Failed to evaluate expression '{}' for subscription: {}", expression, e);
                false
            }
        }
    }
    
    /// 查找通知模板
    /// 根据reason_type和language查找匹配的模板
    async fn find_notification_template(
        &self,
        reason_type: &str,
        language: &str,
    ) -> Option<flow_domain::notification::NotificationTemplate> {
        use flow_api::extension::query::Condition;
        
        // 查找匹配reason_type的模板
        let options = ListOptions {
            condition: Some(Condition::Equal {
                index_name: "spec.reasonSelector.reasonType".to_string(),
                value: serde_json::Value::String(reason_type.to_string()),
            }),
            ..Default::default()
        };
        
        match self.extension_client.list::<flow_domain::notification::NotificationTemplate>(options).await {
            Ok(result) => {
                // 过滤匹配语言的模板，选择最新的（按创建时间）
                let mut matching: Vec<_> = result.items.into_iter()
                    .filter(|t| {
                        t.spec.as_ref()
                            .and_then(|s| s.reason_selector.as_ref())
                            .map(|rs| rs.language.as_str() == language || rs.language == "default")
                            .unwrap_or(false)
                    })
                    .collect();
                
                // 按创建时间排序，选择最新的
                matching.sort_by(|a, b| {
                    let a_time = a.metadata.creation_timestamp.as_ref()
                        .map(|ts| ts.timestamp())
                        .unwrap_or(0);
                    let b_time = b.metadata.creation_timestamp.as_ref()
                        .map(|ts| ts.timestamp())
                        .unwrap_or(0);
                    b_time.cmp(&a_time)
                });
                
                matching.into_iter().next()
            }
            Err(e) => {
                tracing::warn!("Failed to find notification template for reason_type {}: {}", reason_type, e);
                None
            }
        }
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
            
            // 查找通知模板
            let template = self.find_notification_template(&reason.spec.reason_type, "default").await;
            
            // 创建站内通知
            use flow_domain::notification::{Notification, NotificationSpec};
            use flow_api::extension::Metadata;
            
            // 使用模板渲染通知内容（如果找到模板）
            let (title, raw_content, html_content) = if let Some(template) = template {
                if let Some(template_content) = &template.spec.as_ref().and_then(|s| s.template.as_ref()) {
                    // 使用模板内容
                    (
                        template_content.title.clone(),
                        template_content.raw_body.clone().unwrap_or_else(|| "".to_string()),
                        template_content.html_body.clone().unwrap_or_else(|| "".to_string()),
                    )
                } else {
                    // 模板没有内容，使用默认
                    (
                        format!("Notification: {}", reason.spec.subject.title),
                        format!("Reason: {}", reason.spec.reason_type),
                        format!("<p>Reason: {}</p>", reason.spec.reason_type),
                    )
                }
            } else {
                // 没有找到模板，使用默认内容
                (
                    format!("Notification: {}", reason.spec.subject.title),
                    format!("Reason: {}", reason.spec.reason_type),
                    format!("<p>Reason: {}</p>", reason.spec.reason_type),
                )
            };
            
            let notification = Notification {
                metadata: Metadata::new(Uuid::new_v4().to_string()),
                spec: NotificationSpec {
                    recipient: subscriber_name.clone(),
                    reason: reason.metadata.name.clone(),
                    title,
                    raw_content,
                    html_content,
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
        
        let created = subscription.clone();
        self.extension_client.create(subscription).await
            .map_err(|e| anyhow::anyhow!("Failed to create subscription: {}", e))?;
        
        Ok(created)
    }

    async fn unsubscribe(&self, subscriber: &SubscriptionSubscriber) -> Result<()> {
        // 查找该订阅者的所有订阅
        let options = ListOptions {
            condition: Some(Condition::Equal {
                index_name: "spec.subscriber.name".to_string(),
                value: serde_json::Value::String(subscriber.name.clone()),
            }),
            ..Default::default()
        };
        
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
        let options = ListOptions {
            condition: Some(Condition::Equal {
                index_name: "spec.subscriber.name".to_string(),
                value: serde_json::Value::String(subscriber.name.clone()),
            }),
            ..Default::default()
        };
        
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

