pub mod finders;
pub mod installer;

pub use finders::{PostFinder, CategoryFinder, TagFinder, ThemeFinder};

use flow_domain::theme::Theme;
use flow_api::extension::{ExtensionClient, ListOptions};
use flow_infra::extension::ReactiveExtensionClient;
use flow_infra::system_setting::{SystemSettingService, DefaultSystemSettingService};
use async_trait::async_trait;
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::{Result, Context};

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
    system_setting_service: Arc<dyn SystemSettingService>,
    theme_root: PathBuf,
}

impl DefaultThemeService {
    pub fn new(extension_client: Arc<ReactiveExtensionClient>, theme_root: PathBuf) -> Self {
        let system_setting_service: Arc<dyn SystemSettingService> = Arc::new(
            DefaultSystemSettingService::new(extension_client.clone())
        );
        Self {
            extension_client,
            system_setting_service,
            theme_root,
        }
    }
}

#[async_trait]
impl ThemeService for DefaultThemeService {
    async fn get_active_theme(&self) -> Result<Option<String>> {
        let theme_setting = self.system_setting_service.get_theme_setting().await?;
        Ok(theme_setting.and_then(|s| s.active))
    }
    
    async fn set_active_theme(&self, theme_name: &str) -> Result<()> {
        // 验证主题是否存在
        let _theme: Option<Theme> = self.extension_client.fetch(theme_name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch theme: {}", e))?;
        
        if _theme.is_none() {
            anyhow::bail!("Theme not found: {}", theme_name);
        }
        
        // 更新系统设置
        use flow_infra::system_setting::ThemeSetting;
        let setting = ThemeSetting {
            active: Some(theme_name.to_string()),
        };
        
        self.system_setting_service.update_theme_setting(setting).await?;
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
    
    async fn install_theme(&self, content: Vec<u8>) -> Result<Theme> {
        use installer::ThemeInstaller;
        
        // 创建安装器
        let installer = ThemeInstaller::new(self.theme_root.clone());
        
        // 安装主题
        let theme = installer.install_theme(content, false).await?;
        
        // 创建Theme Extension
        self.extension_client.create(theme.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to create theme extension: {}", e))?;
        
        Ok(theme)
    }
    
    async fn upgrade_theme(&self, name: &str, content: Vec<u8>) -> Result<Theme> {
        use installer::ThemeInstaller;
        
        // 创建安装器
        let installer = ThemeInstaller::new(self.theme_root.clone());
        
        // 升级主题
        let theme = installer.upgrade_theme(name, content).await?;
        
        // 更新Theme Extension
        self.extension_client.update(theme.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to update theme extension: {}", e))?;
        
        Ok(theme)
    }
    
    async fn reload_theme(&self, name: &str) -> Result<Theme> {
        // 验证主题是否存在
        let theme_path = self.theme_root.join(name);
        if !theme_path.exists() {
            anyhow::bail!("Theme not found: {}", name);
        }
        
        // 重新解析主题 manifest
        use installer::ThemeInstaller;
        let installer = ThemeInstaller::new(self.theme_root.clone());
        
        // 查找并加载主题manifest
        let theme_manifest_path = installer.locate_theme_manifest(&theme_path)
            .ok_or_else(|| anyhow::anyhow!("Missing theme manifest (theme.yaml or theme.yml)"))?;
        
        let theme = installer.load_theme_manifest(&theme_manifest_path)
            .context("Failed to reload theme manifest")?;
        
        // 验证主题名称是否匹配
        if theme.metadata.name != name {
            anyhow::bail!(
                "Theme name mismatch: expected {}, but got {}",
                name,
                theme.metadata.name
            );
        }
        
        // 更新主题的location字段
        let mut theme_with_location = theme;
        theme_with_location.status = Some(flow_domain::theme::ThemeStatus {
            phase: Some(flow_domain::theme::ThemePhase::Ready),
            conditions: None,
            location: Some(theme_path.to_string_lossy().to_string()),
        });
        
        // 更新Theme Extension（这会触发缓存刷新）
        self.extension_client.update(theme_with_location.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to update theme extension: {}", e))?;
        
        Ok(theme_with_location)
    }
}

