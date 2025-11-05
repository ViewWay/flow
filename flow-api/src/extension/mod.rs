pub mod scheme;
pub mod client;
pub mod index;
pub mod query;

use serde::{Deserialize, Serialize};

/// GroupVersionKind 表示扩展对象的组、版本和类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupVersionKind {
    pub group: String,
    pub version: String,
    pub kind: String,
}

impl GroupVersionKind {
    pub fn new(group: impl Into<String>, version: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            group: group.into(),
            version: version.into(),
            kind: kind.into(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}/{}/{}", self.group, self.version, self.kind)
    }
}

/// Metadata 包含扩展对象的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub version: Option<u64>,
    pub creation_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub labels: Option<std::collections::HashMap<String, String>>,
    pub annotations: Option<std::collections::HashMap<String, String>>,
}

impl Metadata {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: Some(0),
            creation_timestamp: Some(chrono::Utc::now()),
            labels: None,
            annotations: None,
        }
    }
}

/// Extension trait 是所有扩展对象的基础trait
pub trait Extension: Send + Sync {
    fn metadata(&self) -> &Metadata;
    fn group_version_kind(&self) -> GroupVersionKind;
}

/// ListOptions 用于查询扩展对象
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListOptions {
    pub label_selector: Option<String>,
    pub field_selector: Option<String>,
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub sort: Option<Vec<String>>,
    /// 查询条件（如果提供，将覆盖label_selector和field_selector）
    pub condition: Option<query::Condition>,
}

impl ListOptions {
    /// 转换为查询条件
    pub fn to_condition(&self) -> query::Condition {
        self.condition.clone().unwrap_or_else(query::Condition::empty)
    }
}

/// ListResult 包含查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub size: u32,
}

impl<T> ListResult<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, size: u32) -> Self {
        Self {
            items,
            total,
            page,
            size,
        }
    }
}

// ExtensionClient trait 已移动到 client.rs
pub use client::ExtensionClient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_version_kind() {
        let gvk = GroupVersionKind::new("test.group", "v1", "TestKind");
        assert_eq!(gvk.group, "test.group");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "TestKind");
        assert_eq!(gvk.to_string(), "test.group/v1/TestKind");
    }

    #[test]
    fn test_metadata_creation() {
        let metadata = Metadata::new("test-name");
        assert_eq!(metadata.name, "test-name");
        assert_eq!(metadata.version, Some(0));
        assert!(metadata.creation_timestamp.is_some());
    }

    #[test]
    fn test_list_options_default() {
        let options = ListOptions::default();
        assert!(options.label_selector.is_none());
        assert!(options.field_selector.is_none());
        assert!(options.page.is_none());
        assert!(options.size.is_none());
    }

    #[test]
    fn test_list_result() {
        let items = vec![1, 2, 3];
        let result = ListResult::new(items.clone(), 3, 0, 10);
        assert_eq!(result.items, items);
        assert_eq!(result.total, 3);
        assert_eq!(result.page, 0);
        assert_eq!(result.size, 10);
    }
}
