use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use super::constant;

/// Tag实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub metadata: Metadata,
    pub spec: TagSpec,
    pub status: Option<TagStatus>,
}

impl Extension for Tag {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(constant::GROUP, constant::VERSION, constant::TAG_KIND)
    }
}

impl Tag {
    /// 获取状态（如果不存在则返回默认值）
    pub fn status_or_default(&self) -> TagStatus {
        self.status.clone().unwrap_or_default()
    }
}

/// TagSpec包含标签的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSpec {
    #[serde(rename = "displayName")]
    pub display_name: String,
    
    pub slug: String,
    
    /// 颜色（十六进制格式，如：#FF0000 或 #F00）
    pub color: Option<String>,
    
    pub cover: Option<String>,
}

/// TagStatus包含标签的状态信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagStatus {
    pub permalink: Option<String>,
    
    /// 已发布且公开的文章数量
    #[serde(rename = "visiblePostCount")]
    pub visible_post_count: Option<i32>,
    
    /// 文章总数
    #[serde(rename = "postCount")]
    pub post_count: Option<i32>,
    
    #[serde(rename = "observedVersion")]
    pub observed_version: Option<i64>,
}

