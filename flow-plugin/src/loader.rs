use crate::descriptor::PluginDescriptor;
use crate::plugin::Plugin;
use anyhow::Result;
use std::path::Path;
use libloading::{Library, Symbol};

/// 插件加载器trait
pub trait PluginLoader: Send + Sync {
    /// 检查路径是否适用于此加载器
    fn is_applicable(&self, plugin_path: &Path) -> bool;
    
    /// 加载插件描述符
    fn load_descriptor(&self, plugin_path: &Path) -> Result<PluginDescriptor>;
    
    /// 加载插件实例
    fn load_plugin(&self, plugin_path: &Path, descriptor: &PluginDescriptor) -> Result<Box<dyn Plugin>>;
}

/// 动态库插件加载器
/// 用于加载Rust编译的动态库（.so, .dylib, .dll）
pub struct DynamicLibraryLoader;

impl PluginLoader for DynamicLibraryLoader {
    fn is_applicable(&self, plugin_path: &Path) -> bool {
        if let Some(ext) = plugin_path.extension() {
            let ext_str = ext.to_string_lossy();
            matches!(ext_str.as_ref(), "so" | "dylib" | "dll")
        } else {
            false
        }
    }
    
    fn load_descriptor(&self, _plugin_path: &Path) -> Result<PluginDescriptor> {
        // 对于动态库，描述符可能嵌入在库中或单独的元数据文件
        // 这里简化处理，实际应该从库中读取或查找同名的yaml文件
        Err(anyhow::anyhow!("Dynamic library descriptor loading not yet implemented"))
    }
    
    fn load_plugin(&self, plugin_path: &Path, _descriptor: &PluginDescriptor) -> Result<Box<dyn Plugin>> {
        unsafe {
            // 加载动态库
            let lib = Library::new(plugin_path)?;
            
            // 查找插件入口点函数
            // 假设插件导出一个名为 `create_plugin` 的函数
            // 类型签名：extern "C" fn() -> *mut dyn Plugin
            // 注意：这是一个简化的示例，实际实现需要更复杂的类型处理
            
            Err(anyhow::anyhow!("Dynamic library plugin loading not yet fully implemented"))
        }
    }
}

/// 目录插件加载器
/// 用于开发模式，从目录加载插件
pub struct DirectoryPluginLoader;

impl PluginLoader for DirectoryPluginLoader {
    fn is_applicable(&self, plugin_path: &Path) -> bool {
        plugin_path.is_dir()
    }
    
    fn load_descriptor(&self, plugin_path: &Path) -> Result<PluginDescriptor> {
        let descriptor_path = plugin_path.join("plugin.yaml");
        
        // 使用阻塞方式读取（简化实现）
        let content = std::fs::read_to_string(&descriptor_path)
            .map_err(|e| anyhow::anyhow!("Failed to read descriptor: {}", e))?;
        
        PluginDescriptor::from_yaml(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse descriptor: {}", e))
    }
    
    fn load_plugin(&self, _plugin_path: &Path, _descriptor: &PluginDescriptor) -> Result<Box<dyn Plugin>> {
        // 目录插件通常用于开发模式，需要编译后加载
        // 这里返回错误，表示需要先编译
        Err(anyhow::anyhow!("Directory plugin loading requires compilation"))
    }
}

