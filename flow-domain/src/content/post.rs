use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::constant;

/// Post实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub metadata: Metadata,
    pub spec: PostSpec,
    pub status: Option<PostStatus>,
}

impl Extension for Post {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(constant::GROUP, constant::VERSION, constant::POST_KIND)
    }
}

impl Post {
    /// 检查文章是否已删除
    pub fn is_deleted(&self) -> bool {
        self.spec.deleted.unwrap_or(false)
    }

    /// 检查文章是否已发布
    pub fn is_published(&self) -> bool {
        if let Some(labels) = &self.metadata.labels {
            labels.get(constant::POST_PUBLISHED_LABEL)
                .map(|v| v == "true")
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// 检查文章是否公开
    pub fn is_public(&self) -> bool {
        matches!(self.spec.visible, Some(VisibleEnum::Public) | None)
    }

    /// 获取状态（如果不存在则返回默认值）
    pub fn status_or_default(&self) -> PostStatus {
        self.status.clone().unwrap_or_default()
    }
}

/// PostSpec包含文章的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostSpec {
    #[serde(rename = "title")]
    pub title: String,
    
    #[serde(rename = "slug")]
    pub slug: String,
    
    /// 文章引用到的已发布的内容，用于主题端显示
    #[serde(rename = "releaseSnapshot")]
    pub release_snapshot: Option<String>,
    
    #[serde(rename = "headSnapshot")]
    pub head_snapshot: Option<String>,
    
    #[serde(rename = "baseSnapshot")]
    pub base_snapshot: Option<String>,
    
    pub owner: Option<String>,
    
    pub template: Option<String>,
    
    pub cover: Option<String>,
    
    #[serde(default)]
    pub deleted: Option<bool>,
    
    #[serde(default)]
    pub publish: Option<bool>,
    
    #[serde(rename = "publishTime")]
    pub publish_time: Option<DateTime<Utc>>,
    
    #[serde(default)]
    pub pinned: Option<bool>,
    
    #[serde(rename = "allowComment", default = "default_true_option")]
    pub allow_comment: Option<bool>,
    
    #[serde(default)]
    pub visible: Option<VisibleEnum>,
    
    #[serde(default)]
    pub priority: Option<i32>,
    
    pub excerpt: Option<Excerpt>,
    
    pub categories: Option<Vec<String>>,
    
    pub tags: Option<Vec<String>>,
    
    #[serde(rename = "htmlMetas")]
    pub html_metas: Option<Vec<std::collections::HashMap<String, String>>>,
}

fn default_true() -> bool {
    true
}

fn default_true_option() -> Option<bool> {
    Some(true)
}

/// PostStatus包含文章的状态信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostStatus {
    pub phase: Option<PostPhase>,
    
    pub permalink: Option<String>,
    
    pub excerpt: Option<String>,
    
    #[serde(rename = "inProgress")]
    pub in_progress: Option<bool>,
    
    #[serde(rename = "commentsCount")]
    pub comments_count: Option<i32>,
    
    pub contributors: Option<Vec<String>>,
    
    #[serde(rename = "hideFromList")]
    pub hide_from_list: Option<bool>,
    
    #[serde(rename = "lastModifyTime")]
    pub last_modify_time: Option<DateTime<Utc>>,
    
    #[serde(rename = "observedVersion")]
    pub observed_version: Option<i64>,
}

/// PostPhase表示文章的阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PostPhase {
    Draft,
    PendingApproval,
    Published,
    Failed,
}

/// VisibleEnum表示文章的可见性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VisibleEnum {
    Public,
    Internal,
    Private,
}

/// Excerpt表示文章的摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Excerpt {
    #[serde(rename = "autoGenerate", default = "default_true_option")]
    pub auto_generate: Option<bool>,
    
    pub raw: Option<String>,
}

