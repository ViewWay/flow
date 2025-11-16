use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 插件描述符
/// 从插件元数据文件（如plugin.yaml）中读取
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    /// 插件ID（唯一标识符）
    pub id: String,
    
    /// 插件版本（SemVer格式）
    pub version: String,
    
    /// 插件描述
    pub description: Option<String>,
    
    /// 插件提供者（作者）
    pub provider: Option<String>,
    
    /// 插件依赖（插件ID -> 版本要求）
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    
    /// 插件类名（Rust插件的入口点）
    pub plugin_class: Option<String>,
    
    /// 插件库路径（动态库路径）
    pub plugin_lib: Option<String>,
    
    /// Halo版本要求
    #[serde(default = "default_requires")]
    pub requires: String,
    
    /// 许可证
    #[serde(default)]
    pub license: Vec<String>,
}

fn default_requires() -> String {
    "*".to_string()
}

impl PluginDescriptor {
    /// 从YAML字符串解析插件描述符
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
    
    /// 从JSON字符串解析插件描述符
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

