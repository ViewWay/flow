use async_trait::async_trait;
use crate::descriptor::PluginDescriptor;
use anyhow::Result;

/// 插件trait
/// 所有插件必须实现此trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件描述符
    fn descriptor(&self) -> &PluginDescriptor;
    
    /// 启动插件
    /// 在插件加载后调用
    async fn start(&self) -> Result<()>;
    
    /// 停止插件
    /// 在插件卸载前调用
    async fn stop(&self) -> Result<()>;
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// 已创建但未启动
    Created,
    /// 正在启动
    Starting,
    /// 已启动
    Started,
    /// 正在停止
    Stopping,
    /// 已停止
    Stopped,
    /// 失败
    Failed,
}

/// 插件包装器
/// 包装插件实例和状态
pub struct PluginWrapper {
    /// 插件描述符
    pub descriptor: PluginDescriptor,
    
    /// 插件状态
    pub state: PluginState,
    
    /// 插件实例（可选，因为可能使用动态库）
    pub plugin: Option<Box<dyn Plugin>>,
    
    /// 插件路径
    pub plugin_path: String,
}

impl PluginWrapper {
    pub fn new(descriptor: PluginDescriptor, plugin_path: String) -> Self {
        Self {
            descriptor,
            state: PluginState::Created,
            plugin: None,
            plugin_path,
        }
    }
    
    /// 获取插件ID
    pub fn plugin_id(&self) -> &str {
        &self.descriptor.id
    }
    
    /// 获取插件版本
    pub fn version(&self) -> &str {
        &self.descriptor.version
    }
}

