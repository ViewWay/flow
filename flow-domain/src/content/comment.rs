use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::constant;

/// Comment实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub metadata: Metadata,
    pub spec: CommentSpec,
    pub status: Option<CommentStatus>,
}

impl Extension for Comment {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(constant::GROUP, constant::VERSION, constant::COMMENT_KIND)
    }
}

impl Comment {
    /// 获取状态（如果不存在则返回默认值）
    pub fn status_or_default(&self) -> CommentStatus {
        self.status.clone().unwrap_or_default()
    }
}

/// CommentSpec包含评论的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentSpec {
    /// 评论引用的主题（Post或SinglePage）
    #[serde(rename = "subjectRef")]
    pub subject_ref: SubjectRef,
    
    /// 最后阅读时间
    #[serde(rename = "lastReadTime")]
    pub last_read_time: Option<DateTime<Utc>>,
    
    /// 原始内容（Markdown等）
    pub raw: String,
    
    /// 渲染后的内容（HTML）
    pub content: String,
    
    /// 评论所有者
    pub owner: CommentOwner,
    
    /// 用户代理
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
    
    /// IP地址
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    
    /// 批准时间
    #[serde(rename = "approvedTime")]
    pub approved_time: Option<DateTime<Utc>>,
    
    /// 用户定义的创建时间（默认为metadata.creationTimestamp）
    #[serde(rename = "creationTime")]
    pub creation_time: Option<DateTime<Utc>>,
    
    #[serde(default)]
    pub priority: Option<i32>,
    
    #[serde(default)]
    pub top: Option<bool>,
    
    #[serde(rename = "allowNotification", default = "default_true_option")]
    pub allow_notification: Option<bool>,
    
    #[serde(default)]
    pub approved: Option<bool>,
    
    #[serde(default)]
    pub hidden: Option<bool>,
}

fn default_true() -> bool {
    true
}

fn default_true_option() -> Option<bool> {
    Some(true)
}

/// SubjectRef表示评论引用的主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectRef {
    pub group: String,
    pub version: String,
    pub kind: String,
    pub name: String,
}

impl SubjectRef {
    /// 转换为主题引用键
    pub fn to_key(&self) -> String {
        format!("{}/{}/{}", self.group, self.kind, self.name)
    }
}

/// CommentOwner表示评论的所有者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentOwner {
    pub kind: String,
    pub name: String,
    
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    
    pub annotations: Option<std::collections::HashMap<String, String>>,
}

impl CommentOwner {
    pub const KIND_EMAIL: &'static str = "Email";
    
    /// 获取注释值
    pub fn get_annotation(&self, key: &str) -> Option<&String> {
        self.annotations.as_ref()?.get(key)
    }
    
    /// 生成所有者身份标识
    pub fn owner_identity(kind: &str, name: &str) -> String {
        format!("{}#{}", kind, name)
    }
}

/// CommentStatus包含评论的状态信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommentStatus {
    #[serde(rename = "lastReplyTime")]
    pub last_reply_time: Option<DateTime<Utc>>,
    
    #[serde(rename = "replyCount")]
    pub reply_count: Option<i32>,
    
    #[serde(rename = "visibleReplyCount")]
    pub visible_reply_count: Option<i32>,
    
    #[serde(rename = "unreadReplyCount")]
    pub unread_reply_count: Option<i32>,
    
    #[serde(rename = "hasNewReply")]
    pub has_new_reply: Option<bool>,
    
    #[serde(rename = "observedVersion")]
    pub observed_version: Option<i64>,
}

/// BaseCommentSpec是评论的基础规格（用于Reply）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCommentSpec {
    pub raw: String,
    pub content: String,
    pub owner: CommentOwner,
    
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
    
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    
    #[serde(rename = "approvedTime")]
    pub approved_time: Option<DateTime<Utc>>,
    
    #[serde(rename = "creationTime")]
    pub creation_time: Option<DateTime<Utc>>,
    
    #[serde(default)]
    pub priority: Option<i32>,
    
    #[serde(default)]
    pub top: Option<bool>,
    
    #[serde(rename = "allowNotification", default = "default_true_option")]
    pub allow_notification: Option<bool>,
    
    #[serde(default)]
    pub approved: Option<bool>,
    
    #[serde(default)]
    pub hidden: Option<bool>,
}

