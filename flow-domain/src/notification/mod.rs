use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Notification扩展对象
/// 用于存储站内通知信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub metadata: Metadata,
    pub spec: NotificationSpec,
}

impl Extension for Notification {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("notification.halo.run", "v1alpha1", "Notification")
    }
}

/// Notification规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSpec {
    /// 接收者用户名（必需）
    pub recipient: String,
    
    /// 原因名称（必需）
    pub reason: String,
    
    /// 通知标题（必需）
    pub title: String,
    
    /// 原始内容（必需）
    #[serde(rename = "rawContent")]
    pub raw_content: String,
    
    /// HTML内容（必需）
    #[serde(rename = "htmlContent")]
    pub html_content: String,
    
    /// 是否未读
    pub unread: Option<bool>,
    
    /// 最后阅读时间
    #[serde(rename = "lastReadAt")]
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_read_at: Option<DateTime<Utc>>,
}

/// NotificationTemplate扩展对象
/// 定义通知模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub metadata: Metadata,
    pub spec: Option<NotificationTemplateSpec>,
}

impl Extension for NotificationTemplate {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("notification.halo.run", "v1alpha1", "NotificationTemplate")
    }
}

/// NotificationTemplate规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplateSpec {
    /// 原因选择器
    #[serde(rename = "reasonSelector")]
    pub reason_selector: Option<ReasonSelector>,
    
    /// 模板内容
    pub template: Option<TemplateContent>,
}

/// 原因选择器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonSelector {
    /// 原因类型（必需）
    #[serde(rename = "reasonType")]
    pub reason_type: String,
    
    /// 语言（必需，默认"default"）
    pub language: String,
}

/// 模板内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContent {
    /// 标题（必需）
    pub title: String,
    
    /// HTML正文
    #[serde(rename = "htmlBody")]
    pub html_body: Option<String>,
    
    /// 原始正文
    #[serde(rename = "rawBody")]
    pub raw_body: Option<String>,
}

/// Reason扩展对象
/// 定义通知原因（触发通知的事件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reason {
    pub metadata: Metadata,
    pub spec: ReasonSpec,
}

impl Extension for Reason {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("notification.halo.run", "v1alpha1", "Reason")
    }
}

/// Reason规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonSpec {
    /// 原因类型（必需）
    #[serde(rename = "reasonType")]
    pub reason_type: String,
    
    /// 主题（必需）
    pub subject: ReasonSubject,
    
    /// 作者（必需）
    pub author: String,
    
    /// 属性（用于传递数据）
    pub attributes: Option<HashMap<String, String>>,
}

/// Reason主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonSubject {
    /// API版本（必需）
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    
    /// 类型（必需）
    pub kind: String,
    
    /// 名称（必需）
    pub name: String,
    
    /// 标题（必需）
    pub title: String,
    
    /// URL
    pub url: Option<String>,
}

/// Subscription扩展对象
/// 定义订阅（用户订阅某个原因的通知）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub metadata: Metadata,
    pub spec: SubscriptionSpec,
}

impl Extension for Subscription {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("notification.halo.run", "v1alpha1", "Subscription")
    }
}

/// Subscription规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSpec {
    /// 订阅者（必需）
    pub subscriber: SubscriptionSubscriber,
    
    /// 取消订阅token（必需）
    #[serde(rename = "unsubscribeToken")]
    pub unsubscribe_token: String,
    
    /// 感兴趣的原因（必需）
    pub reason: InterestReason,
    
    /// 是否禁用
    pub disabled: Option<bool>,
}

/// 订阅者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSubscriber {
    /// 订阅者名称（必需）
    pub name: String,
}

/// 感兴趣的原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestReason {
    /// 原因类型名称（必需）
    #[serde(rename = "reasonType")]
    pub reason_type: String,
    
    /// 原因主题（必需）
    pub subject: Option<InterestReasonSubject>,
    
    /// 表达式（可选）
    pub expression: Option<String>,
}

/// 感兴趣的原因主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestReasonSubject {
    /// 名称（可选，如果不指定，表示所有主题）
    pub name: Option<String>,
    
    /// API版本（必需）
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    
    /// 类型（必需）
    pub kind: String,
}

impl Subscription {
    /// 生成取消订阅token
    pub fn generate_unsubscribe_token() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

