use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// HaloDocument 搜索文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaloDocument {
    /// 文档ID，全局唯一
    pub id: String,
    
    /// 对应扩展对象的metadata name
    pub metadata_name: String,
    
    /// 自定义元数据
    pub annotations: Option<HashMap<String, String>>,
    
    /// 文档标题
    pub title: String,
    
    /// 文档描述
    pub description: Option<String>,
    
    /// 文档内容（安全内容，无HTML标签）
    pub content: String,
    
    /// 文档分类（分类的metadata name列表）
    pub categories: Option<Vec<String>>,
    
    /// 文档标签（标签的metadata name列表）
    pub tags: Option<Vec<String>>,
    
    /// 是否已发布
    pub published: bool,
    
    /// 是否已回收
    pub recycled: bool,
    
    /// 是否公开暴露
    pub exposed: bool,
    
    /// 文档所有者metadata name
    pub owner_name: String,
    
    /// 文档创建时间戳
    pub creation_timestamp: Option<DateTime<Utc>>,
    
    /// 文档更新时间戳
    pub update_timestamp: Option<DateTime<Utc>>,
    
    /// 文档永久链接
    pub permalink: String,
    
    /// 文档类型，例如：post.content.halo.run, singlepage.content.halo.run
    pub doc_type: String,
}

/// SearchOption 搜索选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOption {
    /// 搜索关键词
    pub keyword: String,
    
    /// 结果限制
    #[serde(default = "default_limit")]
    pub limit: u32,
    
    /// 高亮片段的前HTML标签
    #[serde(default = "default_highlight_pre_tag")]
    pub highlight_pre_tag: String,
    
    /// 高亮片段的后HTML标签
    #[serde(default = "default_highlight_post_tag")]
    pub highlight_post_tag: String,
    
    /// 是否过滤公开内容（None表示不过滤）
    pub filter_exposed: Option<bool>,
    
    /// 是否过滤回收内容（None表示不过滤）
    pub filter_recycled: Option<bool>,
    
    /// 是否过滤已发布内容（None表示不过滤）
    pub filter_published: Option<bool>,
    
    /// 要包含的类型（OR关系，None表示包含所有类型）
    pub include_types: Option<Vec<String>>,
    
    /// 要包含的所有者（OR关系，None表示包含所有所有者）
    pub include_owner_names: Option<Vec<String>>,
    
    /// 要包含的分类（AND关系，None表示包含所有分类）
    pub include_category_names: Option<Vec<String>>,
    
    /// 要包含的标签（AND关系，None表示包含所有标签）
    pub include_tag_names: Option<Vec<String>>,
    
    /// 额外的注解（用于扩展搜索选项）
    pub annotations: Option<HashMap<String, String>>,
}

fn default_limit() -> u32 {
    10
}

fn default_highlight_pre_tag() -> String {
    "<B>".to_string()
}

fn default_highlight_post_tag() -> String {
    "</B>".to_string()
}

/// SearchResult 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 搜索结果列表
    pub hits: Vec<HaloDocument>,
    
    /// 搜索关键词
    pub keyword: String,
    
    /// 总结果数
    pub total: u64,
    
    /// 结果限制
    pub limit: u32,
    
    /// 处理时间（毫秒）
    pub processing_time_millis: u64,
}

/// SearchEngine trait 定义搜索引擎接口
#[async_trait::async_trait]
pub trait SearchEngine: Send + Sync {
    /// 搜索引擎是否可用
    fn available(&self) -> bool;
    
    /// 添加或更新Halo文档
    async fn add_or_update(&self, documents: Vec<HaloDocument>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 根据ID删除文档
    async fn delete_document(&self, doc_ids: Vec<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 删除所有文档
    async fn delete_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 搜索文档
    async fn search(&self, option: SearchOption) -> Result<SearchResult, Box<dyn std::error::Error + Send + Sync>>;
}

