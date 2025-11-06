use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use super::constant;

/// Category实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub metadata: Metadata,
    pub spec: CategorySpec,
    pub status: Option<CategoryStatus>,
}

impl Extension for Category {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(constant::GROUP, constant::VERSION, constant::CATEGORY_KIND)
    }
}

impl Category {
    /// 检查分类是否已删除（通过deletionTimestamp注解）
    pub fn is_deleted(&self) -> bool {
        // 在Halo中，删除通过deletionTimestamp注解标记
        // 这里我们简化处理，实际应该检查metadata中的deletionTimestamp
        false // TODO: 实现deletionTimestamp检查
    }

    /// 获取状态（如果不存在则返回默认值）
    pub fn status_or_default(&self) -> CategoryStatus {
        self.status.clone().unwrap_or_default()
    }
}

/// CategorySpec包含分类的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySpec {
    #[serde(rename = "displayName")]
    pub display_name: String,
    
    pub slug: String,
    
    pub description: Option<String>,
    
    pub cover: Option<String>,
    
    pub template: Option<String>,
    
    /// 用于指定该分类下文章使用的模板
    #[serde(rename = "postTemplate")]
    pub post_template: Option<String>,
    
    #[serde(default)]
    pub priority: Option<i32>,
    
    /// 子分类列表
    pub children: Option<Vec<String>>,
    
    /// 是否阻止父级级联查询
    /// 如果设置为true，查询该分类下的文章时，不会查询子分类的文章
    #[serde(rename = "preventParentPostCascadeQuery")]
    pub prevent_parent_post_cascade_query: Option<bool>,
    
    /// 是否从分类列表中隐藏
    /// 当设置为true时，该分类及其子分类和相关文章不会显示在分类列表中，但仍可通过永久链接访问
    /// 限制：仅在主题端分类列表中生效，且只能在第一级（根节点）分类上设置为true
    #[serde(rename = "hideFromList")]
    pub hide_from_list: Option<bool>,
}

/// CategoryStatus包含分类的状态信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryStatus {
    pub permalink: Option<String>,
    
    /// 包括当前和其下所有层级的文章数量 (depth=max)
    #[serde(rename = "postCount")]
    pub post_count: Option<i32>,
    
    /// 包括当前和其下所有层级的已发布且公开的文章数量 (depth=max)
    #[serde(rename = "visiblePostCount")]
    pub visible_post_count: Option<i32>,
}

