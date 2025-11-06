pub mod finders;

use flow_domain::theme::Theme;
use flow_api::extension::{ExtensionClient, ListOptions};
use flow_infra::extension::ReactiveExtensionClient;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

/// Theme服务trait
#[async_trait]
pub trait ThemeService: Send + Sync {
    /// 获取激活的主题名称
    async fn get_active_theme(&self) -> Result<Option<String>>;
    
    /// 设置激活的主题
    async fn set_active_theme(&self, theme_name: &str) -> Result<()>;
    
    /// 获取主题
    async fn get_theme(&self, name: &str) -> Result<Option<Theme>>;
    
    /// 列出所有主题
    async fn list_themes(&self, options: ListOptions) -> Result<Vec<Theme>>;
    
    /// 安装主题（从ZIP文件）
    async fn install_theme(&self, content: Vec<u8>) -> Result<Theme>;
    
    /// 升级主题
    async fn upgrade_theme(&self, name: &str, content: Vec<u8>) -> Result<Theme>;
    
    /// 重新加载主题
    async fn reload_theme(&self, name: &str) -> Result<Theme>;
}

/// 默认Theme服务实现
pub struct DefaultThemeService {
    extension_client: Arc<ReactiveExtensionClient>,
}

impl DefaultThemeService {
    pub fn new(extension_client: Arc<ReactiveExtensionClient>) -> Self {
        Self { extension_client }
    }
}

#[async_trait]
impl ThemeService for DefaultThemeService {
    async fn get_active_theme(&self) -> Result<Option<String>> {
        // TODO: 从SystemSetting中获取激活的主题
        // 当前简化实现，返回None
        Ok(None)
    }
    
    async fn set_active_theme(&self, theme_name: &str) -> Result<()> {
        // TODO: 更新SystemSetting中的激活主题
        // 当前简化实现
        Ok(())
    }
    
    async fn get_theme(&self, name: &str) -> Result<Option<Theme>> {
        self.extension_client.fetch(name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch theme: {}", e))
    }
    
    async fn list_themes(&self, options: ListOptions) -> Result<Vec<Theme>> {
        let result = self.extension_client.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list themes: {}", e))?;
        Ok(result.items)
    }
    
    async fn install_theme(&self, _content: Vec<u8>) -> Result<Theme> {
        // TODO: 实现主题安装逻辑（解压ZIP、验证、创建Theme Extension）
        anyhow::bail!("Theme installation not implemented yet")
    }
    
    async fn upgrade_theme(&self, _name: &str, _content: Vec<u8>) -> Result<Theme> {
        // TODO: 实现主题升级逻辑
        anyhow::bail!("Theme upgrade not implemented yet")
    }
    
    async fn reload_theme(&self, name: &str) -> Result<Theme> {
        // 重新获取主题
        self.get_theme(name).await?
            .ok_or_else(|| anyhow::anyhow!("Theme not found: {}", name))
    }
}

