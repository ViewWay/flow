use async_trait::async_trait;
use flow_domain::notification::{Reason, Subscription, SubscriptionSubscriber, InterestReason};
use std::sync::Arc;
use anyhow::Result;
use std::collections::HashMap;
use serde_json::Value;

pub mod notification_service;
pub mod notification_center;

pub use notification_service::{NotificationService, DefaultNotificationService};

/// 通知上下文
/// 包含发送通知所需的所有信息
#[derive(Debug, Clone)]
pub struct NotificationContext {
    /// 消息内容
    pub message: NotificationMessage,
    
    /// 接收者配置
    pub receiver_config: Option<HashMap<String, Value>>,
    
    /// 发送者配置
    pub sender_config: Option<HashMap<String, Value>>,
}

/// 通知消息
#[derive(Debug, Clone)]
pub struct NotificationMessage {
    /// 消息负载
    pub payload: MessagePayload,
    
    /// 主题
    pub subject: NotificationSubject,
    
    /// 接收者
    pub recipient: String,
    
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 消息负载
#[derive(Debug, Clone)]
pub struct MessagePayload {
    /// 标题
    pub title: String,
    
    /// 原始正文
    pub raw_body: Option<String>,
    
    /// HTML正文
    pub html_body: Option<String>,
    
    /// 属性
    pub attributes: Option<HashMap<String, String>>,
}

/// 通知主题
#[derive(Debug, Clone)]
pub struct NotificationSubject {
    /// API版本
    pub api_version: String,
    
    /// 类型
    pub kind: String,
    
    /// 名称
    pub name: String,
    
    /// 标题
    pub title: String,
    
    /// URL
    pub url: Option<String>,
}

/// 通知发送器trait
/// 用于发送通知（可以是站内通知、邮件、短信等）
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// 发送通知
    /// 
    /// # 参数
    /// - `notifier_extension_name`: 通知器扩展名称
    /// - `context`: 通知上下文
    async fn send_notification(
        &self,
        notifier_extension_name: &str,
        context: NotificationContext,
    ) -> Result<()>;
}

/// 通知中心trait
/// 负责通知的发送和管理
#[async_trait]
pub trait NotificationCenter: Send + Sync {
    /// 发送通知（基于Reason）
    /// 
    /// # 参数
    /// - `reason`: 通知原因
    async fn notify(&self, reason: Reason) -> Result<()>;
    
    /// 订阅通知
    /// 
    /// # 参数
    /// - `subscriber`: 订阅者
    /// - `interest_reason`: 感兴趣的原因
    /// 
    /// # 返回
    /// - 创建的Subscription
    async fn subscribe(
        &self,
        subscriber: SubscriptionSubscriber,
        interest_reason: InterestReason,
    ) -> Result<Subscription>;
    
    /// 取消订阅（所有原因）
    /// 
    /// # 参数
    /// - `subscriber`: 订阅者
    async fn unsubscribe(&self, subscriber: &SubscriptionSubscriber) -> Result<()>;
    
    /// 取消订阅（特定原因）
    /// 
    /// # 参数
    /// - `subscriber`: 订阅者
    /// - `interest_reason`: 要取消订阅的原因
    async fn unsubscribe_reason(
        &self,
        subscriber: &SubscriptionSubscriber,
        interest_reason: &InterestReason,
    ) -> Result<()>;
}

pub use notification_center::DefaultNotificationCenter;
