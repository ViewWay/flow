use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Attachment扩展对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub metadata: Metadata,
    pub spec: AttachmentSpec,
    pub status: Option<AttachmentStatus>,
}

impl Extension for Attachment {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("storage.halo.run", "v1alpha1", "Attachment")
    }
}

/// Attachment规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentSpec {
    /// 显示名称
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    
    /// 组名
    #[serde(rename = "groupName")]
    pub group_name: Option<String>,
    
    /// 策略名称
    #[serde(rename = "policyName")]
    pub policy_name: Option<String>,
    
    /// 上传者用户名
    #[serde(rename = "ownerName")]
    pub owner_name: Option<String>,
    
    /// 媒体类型
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    
    /// 文件大小（字节）
    pub size: Option<u64>,
    
    /// 标签
    pub tags: Option<Vec<String>>,
}

/// Attachment状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentStatus {
    /// 永久链接（permalink）
    pub permalink: Option<String>,
    
    /// 缩略图链接（key为缩略图尺寸：XL, L, M, S）
    pub thumbnails: Option<HashMap<String, String>>,
}

/// PolicyTemplate扩展对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTemplate {
    pub metadata: Metadata,
    pub spec: Option<PolicyTemplateSpec>,
}

impl Extension for PolicyTemplate {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("storage.halo.run", "v1alpha1", "PolicyTemplate")
    }
}

/// PolicyTemplate规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTemplateSpec {
    /// 显示名称
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    
    /// 设置名称（必需）
    #[serde(rename = "settingName")]
    pub setting_name: String,
}

/// Policy扩展对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub metadata: Metadata,
    pub spec: PolicySpec,
}

impl Extension for Policy {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("storage.halo.run", "v1alpha1", "Policy")
    }
}

/// Policy规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySpec {
    /// 显示名称（必需）
    #[serde(rename = "displayName")]
    pub display_name: String,
    
    /// PolicyTemplate引用名称（必需）
    #[serde(rename = "templateName")]
    pub template_name: String,
    
    /// ConfigMap引用名称
    #[serde(rename = "configMapName")]
    pub config_map_name: Option<String>,
}

/// Group扩展对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub metadata: Metadata,
    pub spec: GroupSpec,
    pub status: Option<GroupStatus>,
}

impl Extension for Group {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("storage.halo.run", "v1alpha1", "Group")
    }
}

/// Group规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSpec {
    /// 显示名称（必需）
    #[serde(rename = "displayName")]
    pub display_name: String,
}

/// Group状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStatus {
    /// 更新时间戳
    #[serde(rename = "updateTimestamp")]
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub update_timestamp: Option<DateTime<Utc>>,
    
    /// 该分组下的附件总数
    #[serde(rename = "totalAttachments")]
    pub total_attachments: Option<u64>,
}

/// 缩略图尺寸
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ThumbnailSize {
    /// 超大尺寸（1600px）
    Xl,
    /// 大尺寸（1200px）
    L,
    /// 中等尺寸（800px）
    M,
    /// 小尺寸（400px）
    S,
}

impl ThumbnailSize {
    /// 获取宽度（像素）
    pub fn width(&self) -> u32 {
        match self {
            ThumbnailSize::Xl => 1600,
            ThumbnailSize::L => 1200,
            ThumbnailSize::M => 800,
            ThumbnailSize::S => 400,
        }
    }
    
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "XL" => Some(ThumbnailSize::Xl),
            "L" => Some(ThumbnailSize::L),
            "M" => Some(ThumbnailSize::M),
            "S" => Some(ThumbnailSize::S),
            _ => None,
        }
    }
    
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ThumbnailSize::Xl => "XL",
            ThumbnailSize::L => "L",
            ThumbnailSize::M => "M",
            ThumbnailSize::S => "S",
        }
    }
}

