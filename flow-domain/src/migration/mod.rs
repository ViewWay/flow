use serde::{Deserialize, Serialize};
use flow_api::extension::{GroupVersionKind, Metadata};
use chrono::{DateTime, Utc};

/// Backup扩展对象
/// 定义备份操作和状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub metadata: Metadata,
    pub spec: BackupSpec,
    pub status: BackupStatus,
}

impl flow_api::extension::Extension for Backup {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("migration.halo.run", "v1alpha1", "Backup")
    }
}

/// Backup规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSpec {
    /// 备份文件格式（目前仅支持zip）
    #[serde(default = "default_format")]
    pub format: String,
    
    /// 自动删除时间
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_format() -> String {
    "zip".to_string()
}

/// Backup状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    /// 备份阶段
    #[serde(default)]
    pub phase: BackupPhase,
    
    /// 开始时间戳
    pub start_timestamp: Option<DateTime<Utc>>,
    
    /// 完成时间戳
    pub completion_timestamp: Option<DateTime<Utc>>,
    
    /// 失败原因（机器可识别）
    pub failure_reason: Option<String>,
    
    /// 失败消息（人类可读）
    pub failure_message: Option<String>,
    
    /// 备份文件大小（字节）
    pub size: Option<u64>,
    
    /// 备份文件名
    pub filename: Option<String>,
}

/// 备份阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BackupPhase {
    /// 等待处理
    Pending,
    /// 正在运行
    Running,
    /// 成功完成
    Succeeded,
    /// 失败
    Failed,
}

impl Default for BackupPhase {
    fn default() -> Self {
        Self::Pending
    }
}

/// 备份文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
    /// 文件名
    pub filename: String,
    
    /// 文件大小（字节）
    pub size: u64,
    
    /// 最后修改时间
    pub last_modified: DateTime<Utc>,
}


