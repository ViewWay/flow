use async_trait::async_trait;
use crate::plugin::{Plugin, PluginWrapper, PluginState};
use crate::descriptor::PluginDescriptor;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use anyhow::Result;
use std::path::PathBuf;

/// 插件管理器trait
#[async_trait]
pub trait PluginManager: Send + Sync {
    /// 加载插件
    /// 
    /// # 参数
    /// - `plugin_path`: 插件路径（目录或文件）
    async fn load_plugin(&self, plugin_path: PathBuf) -> Result<String>;
    
    /// 启动插件
    /// 
    /// # 参数
    /// - `plugin_id`: 插件ID
    async fn start_plugin(&self, plugin_id: &str) -> Result<()>;
    
    /// 停止插件
    /// 
    /// # 参数
    /// - `plugin_id`: 插件ID
    async fn stop_plugin(&self, plugin_id: &str) -> Result<()>;
    
    /// 卸载插件
    /// 
    /// # 参数
    /// - `plugin_id`: 插件ID
    async fn unload_plugin(&self, plugin_id: &str) -> Result<()>;
    
    /// 获取插件
    /// 
    /// # 参数
    /// - `plugin_id`: 插件ID
    /// 
    /// # 返回
    /// - 插件包装器（如果存在）
    async fn get_plugin(&self, plugin_id: &str) -> Option<Arc<PluginWrapper>>;
    
    /// 获取所有已加载的插件
    async fn get_plugins(&self) -> Vec<Arc<PluginWrapper>>;
    
    /// 获取已启动的插件列表
    async fn get_started_plugins(&self) -> Vec<Arc<PluginWrapper>>;
}

/// 默认插件管理器实现
pub struct DefaultPluginManager {
    /// 插件存储：插件ID -> 插件包装器
    plugins: Arc<RwLock<HashMap<String, Arc<PluginWrapper>>>>,
    
    /// 插件根目录
    plugins_root: PathBuf,
}

impl DefaultPluginManager {
    pub fn new(plugins_root: PathBuf) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugins_root,
        }
    }
    
    /// 扫描插件目录并加载所有插件
    pub async fn scan_and_load(&self) -> Result<()> {
        use tokio::fs;
        
        if !self.plugins_root.exists() {
            fs::create_dir_all(&self.plugins_root).await?;
            return Ok(());
        }
        
        let mut entries = fs::read_dir(&self.plugins_root).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() || (path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar")) {
                // 尝试加载插件
                if let Err(e) = self.load_plugin(path).await {
                    eprintln!("Failed to load plugin from {:?}: {}", entry.path(), e);
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl PluginManager for DefaultPluginManager {
    async fn load_plugin(&self, plugin_path: PathBuf) -> Result<String> {
        use tokio::fs;
        use crate::loader::PluginLoader;
        
        // 查找插件描述符文件
        let descriptor_path = if plugin_path.is_dir() {
            plugin_path.join("plugin.yaml")
        } else {
            // 对于JAR文件，需要解压或使用其他方式读取
            return Err(anyhow::anyhow!("JAR plugin loading not yet implemented"));
        };
        
        if !descriptor_path.exists() {
            return Err(anyhow::anyhow!("Plugin descriptor not found: {:?}", descriptor_path));
        }
        
        // 读取并解析描述符
        let yaml_content = fs::read_to_string(&descriptor_path).await?;
        let descriptor = PluginDescriptor::from_yaml(&yaml_content)?;
        
        let plugin_id = descriptor.id.clone();
        
        // 创建插件包装器
        let wrapper = Arc::new(PluginWrapper::new(
            descriptor,
            plugin_path.to_string_lossy().to_string(),
        ));
        
        // 存储插件
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id.clone(), wrapper);
        
        Ok(plugin_id)
    }
    
    async fn start_plugin(&self, plugin_id: &str) -> Result<()> {
        let plugins = self.plugins.read().await;
        let wrapper = plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;
        
        // 更新状态为Starting
        // 注意：由于Arc的不可变性，我们需要重新设计这部分
        // 这里先简化实现
        
        if let Some(plugin) = &wrapper.plugin {
            plugin.start().await?;
        }
        
        Ok(())
    }
    
    async fn stop_plugin(&self, plugin_id: &str) -> Result<()> {
        let plugins = self.plugins.read().await;
        let wrapper = plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;
        
        if let Some(plugin) = &wrapper.plugin {
            plugin.stop().await?;
        }
        
        Ok(())
    }
    
    async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        // 先停止插件
        self.stop_plugin(plugin_id).await?;
        
        // 从存储中移除
        let mut plugins = self.plugins.write().await;
        plugins.remove(plugin_id);
        
        Ok(())
    }
    
    async fn get_plugin(&self, plugin_id: &str) -> Option<Arc<PluginWrapper>> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).cloned()
    }
    
    async fn get_plugins(&self) -> Vec<Arc<PluginWrapper>> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }
    
    async fn get_started_plugins(&self) -> Vec<Arc<PluginWrapper>> {
        let plugins = self.plugins.read().await;
        plugins.values()
            .filter(|w| matches!(w.state, PluginState::Started))
            .cloned()
            .collect()
    }
}

