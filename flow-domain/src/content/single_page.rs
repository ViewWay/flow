use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use super::post::{VisibleEnum, Excerpt};
use super::constant;

/// SinglePage实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePage {
    pub metadata: Metadata,
    pub spec: SinglePageSpec,
    pub status: Option<SinglePageStatus>,
}

impl Extension for SinglePage {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(constant::GROUP, constant::VERSION, constant::SINGLE_PAGE_KIND)
    }
}

impl SinglePage {
    /// 检查单页是否已发布
    pub fn is_published(&self) -> bool {
        if let Some(labels) = &self.metadata.labels {
            labels.get(constant::POST_PUBLISHED_LABEL)
                .map(|v| v == "true")
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// 获取状态（如果不存在则返回默认值）
    pub fn status_or_default(&self) -> SinglePageStatus {
        self.status.clone().unwrap_or_default()
    }
}

/// SinglePageSpec包含单页的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePageSpec {
    pub title: String,
    pub slug: String,
    
    /// 引用到的已发布的内容，用于主题端显示
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
    pub publish_time: Option<chrono::DateTime<chrono::Utc>>,
    
    #[serde(default)]
    pub pinned: Option<bool>,
    
    #[serde(rename = "allowComment", default = "default_true_option")]
    pub allow_comment: Option<bool>,
    
    #[serde(default)]
    pub visible: Option<VisibleEnum>,
    
    #[serde(default)]
    pub priority: Option<i32>,
    
    pub excerpt: Option<Excerpt>,
    
    #[serde(rename = "htmlMetas")]
    pub html_metas: Option<Vec<std::collections::HashMap<String, String>>>,
}

fn default_true() -> bool {
    true
}

fn default_true_option() -> Option<bool> {
    Some(true)
}

/// SinglePageStatus包含单页的状态信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SinglePageStatus {
    pub phase: Option<super::post::PostPhase>,
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
    pub last_modify_time: Option<chrono::DateTime<chrono::Utc>>,
    
    #[serde(rename = "observedVersion")]
    pub observed_version: Option<i64>,
}

