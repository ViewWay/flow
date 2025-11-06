use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::constant;
use super::comment::SubjectRef;

/// Snapshot实体（内容快照，用于版本控制）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub metadata: Metadata,
    pub spec: SnapshotSpec,
}

impl Extension for Snapshot {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(constant::GROUP, constant::VERSION, constant::SNAPSHOT_KIND)
    }
}

impl Snapshot {
    /// 检查是否是基础快照
    pub fn is_base_snapshot(&self) -> bool {
        if let Some(annotations) = &self.metadata.annotations {
            annotations.get(constant::SNAPSHOT_KEEP_RAW_ANNO)
                .map(|v| v == "true")
                .unwrap_or(false)
        } else {
            false
        }
    }
    
    /// 添加贡献者
    pub fn add_contributor(&mut self, name: String) {
        if self.spec.contributors.is_none() {
            self.spec.contributors = Some(Vec::new());
        }
        if let Some(ref mut contributors) = self.spec.contributors {
            if !contributors.contains(&name) {
                contributors.push(name);
            }
        }
    }
}

/// SnapshotSpec包含快照的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotSpec {
    /// 快照引用的主题（Post或SinglePage）
    #[serde(rename = "subjectRef")]
    pub subject_ref: SubjectRef,
    
    /// 原始内容类型（如：markdown | html | json | asciidoc | latex）
    #[serde(rename = "rawType")]
    pub raw_type: String,
    
    /// 原始内容的补丁（diff）
    #[serde(rename = "rawPatch")]
    pub raw_patch: Option<String>,
    
    /// 渲染后内容的补丁（diff）
    #[serde(rename = "contentPatch")]
    pub content_patch: Option<String>,
    
    /// 父快照名称
    #[serde(rename = "parentSnapshotName")]
    pub parent_snapshot_name: Option<String>,
    
    /// 最后修改时间
    #[serde(rename = "lastModifyTime")]
    pub last_modify_time: Option<DateTime<Utc>>,
    
    /// 所有者
    pub owner: String,
    
    /// 贡献者列表
    pub contributors: Option<Vec<String>>,
}

impl SnapshotSpec {
    /// 转换为主题引用键
    pub fn to_subject_ref_key(&self) -> String {
        self.subject_ref.to_key()
    }
}

