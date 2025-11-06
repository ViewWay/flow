use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Theme扩展对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub metadata: Metadata,
    pub spec: ThemeSpec,
    pub status: Option<ThemeStatus>,
}

impl Extension for Theme {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("theme.halo.run", "v1alpha1", "Theme")
    }
}

/// Theme规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpec {
    /// 显示名称
    pub display_name: String,
    
    /// 作者信息
    pub author: Author,
    
    /// 描述
    pub description: Option<String>,
    
    /// Logo
    pub logo: Option<String>,
    
    /// 主页
    pub homepage: Option<String>,
    
    /// 仓库地址
    pub repo: Option<String>,
    
    /// Issues地址
    pub issues: Option<String>,
    
    /// 版本
    #[serde(default = "default_wildcard")]
    pub version: String,
    
    /// 要求的最低Halo版本
    #[serde(default = "default_wildcard")]
    pub requires: String,
    
    /// 设置名称
    pub setting_name: Option<String>,
    
    /// ConfigMap名称
    pub config_map_name: Option<String>,
    
    /// 许可证列表
    pub license: Option<Vec<License>>,
    
    /// 自定义模板
    pub custom_templates: Option<CustomTemplates>,
}

fn default_wildcard() -> String {
    "*".to_string()
}

/// 作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// 作者名称
    pub name: String,
    
    /// 作者网站
    pub website: Option<String>,
}

/// 许可证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

/// 自定义模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTemplates {
    pub post: Option<Vec<TemplateDescriptor>>,
    pub category: Option<Vec<TemplateDescriptor>>,
    pub page: Option<Vec<TemplateDescriptor>>,
}

/// 模板描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDescriptor {
    /// 模板名称
    pub name: String,
    
    /// 描述
    pub description: Option<String>,
    
    /// 截图
    pub screenshot: Option<String>,
    
    /// 模板文件路径
    pub file: String,
}

/// Theme状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeStatus {
    /// 阶段
    pub phase: Option<ThemePhase>,
    
    /// 条件列表
    pub conditions: Option<Vec<Condition>>,
    
    /// 位置（主题文件路径）
    pub location: Option<String>,
}

/// Theme阶段
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ThemePhase {
    Ready,
    Failed,
    Unknown,
}

/// 条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub r#type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_transition_time: Option<DateTime<Utc>>,
}

/// Theme上下文（用于模板渲染）
#[derive(Debug, Clone)]
pub struct ThemeContext {
    /// 主题名称
    pub name: String,
    
    /// 主题路径
    pub path: std::path::PathBuf,
    
    /// 是否激活
    pub active: bool,
}

impl ThemeContext {
    pub fn new(name: String, path: std::path::PathBuf, active: bool) -> Self {
        Self { name, path, active }
    }
}

