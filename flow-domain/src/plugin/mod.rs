use serde::{Deserialize, Serialize};
use flow_api::extension::{GroupVersionKind, Metadata};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Plugin扩展对象
/// 定义插件资源和管理状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub metadata: Metadata,
    pub spec: PluginSpec,
    pub status: Option<PluginStatus>,
}

impl flow_api::extension::Extension for Plugin {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("plugin.halo.run", "v1alpha1", "Plugin")
    }
}

/// Plugin规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSpec {
    /// 显示名称
    pub display_name: Option<String>,
    
    /// 插件版本（SemVer格式）
    pub version: String,
    
    /// 作者信息
    pub author: Option<PluginAuthor>,
    
    /// Logo URL
    pub logo: Option<String>,
    
    /// 插件依赖（插件名 -> 版本要求）
    #[serde(default)]
    pub plugin_dependencies: HashMap<String, String>,
    
    /// 主页URL
    pub homepage: Option<String>,
    
    /// 仓库URL
    pub repo: Option<String>,
    
    /// Issues URL
    pub issues: Option<String>,
    
    /// 描述
    pub description: Option<String>,
    
    /// 许可证列表
    #[serde(default)]
    pub license: Vec<License>,
    
    /// Halo版本要求（SemVer格式）
    #[serde(default = "default_requires")]
    pub requires: String,
    
    /// 是否启用
    #[serde(default)]
    pub enabled: bool,
    
    /// 设置名称
    pub setting_name: Option<String>,
    
    /// 配置Map名称
    pub config_map_name: Option<String>,
}

fn default_requires() -> String {
    "*".to_string()
}

/// 许可证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

/// 插件作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    pub name: String,
    pub website: Option<String>,
}

/// Plugin状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    /// 插件阶段
    #[serde(default)]
    pub phase: PluginPhase,
    
    /// 最后启动时间
    pub last_start_time: Option<DateTime<Utc>>,
    
    /// 最后探测状态
    pub last_probe_state: Option<String>,
    
    /// 入口文件路径
    pub entry: Option<String>,
    
    /// 样式表路径
    pub stylesheet: Option<String>,
    
    /// Logo路径
    pub logo: Option<String>,
    
    /// 加载位置（通常是路径）
    pub load_location: Option<String>,
}

/// 插件阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PluginPhase {
    /// 等待处理
    Pending,
    /// 正在启动
    Starting,
    /// 已创建
    Created,
    /// 正在禁用
    Disabling,
    /// 已禁用
    Disabled,
    /// 已解析
    Resolved,
    /// 已启动
    Started,
    /// 已停止
    Stopped,
    /// 失败
    Failed,
    /// 未知
    Unknown,
}

impl Default for PluginPhase {
    fn default() -> Self {
        Self::Pending
    }
}

