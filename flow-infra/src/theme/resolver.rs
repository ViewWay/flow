use flow_domain::theme::ThemeContext;
use flow_api::extension::ExtensionClient;
use crate::extension::ReactiveExtensionClient;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;

/// 主题解析器
/// 负责解析当前请求应该使用的主题
pub struct ThemeResolver {
    extension_client: Arc<ReactiveExtensionClient>,
    theme_root: PathBuf,
    active_theme: Arc<tokio::sync::RwLock<Option<String>>>,
}

impl ThemeResolver {
    pub fn new(
        extension_client: Arc<ReactiveExtensionClient>,
        theme_root: PathBuf,
    ) -> Self {
        Self {
            extension_client,
            theme_root,
            active_theme: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }
    
    /// 设置激活的主题
    pub async fn set_active_theme(&self, theme_name: &str) {
        let mut active = self.active_theme.write().await;
        *active = Some(theme_name.to_string());
    }
    
    /// 获取激活的主题名称
    pub async fn get_active_theme(&self) -> Option<String> {
        let active = self.active_theme.read().await;
        active.clone()
    }
    
    /// 获取主题上下文
    pub async fn get_theme_context(&self, theme_name: &str) -> Result<ThemeContext> {
        // 验证主题是否存在
        use flow_domain::theme::Theme;
        use flow_api::extension::Extension;
        let _theme: Option<Theme> = self.extension_client.fetch(theme_name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch theme: {}", e))?;
        
        if _theme.is_none() {
            anyhow::bail!("Theme not found: {}", theme_name);
        }
        
        let active_theme = self.get_active_theme().await;
        let active = active_theme.as_ref().map(|name| name == theme_name).unwrap_or(false);
        
        let path = self.theme_root.join(theme_name);
        
        Ok(ThemeContext::new(theme_name.to_string(), path, active))
    }
    
    /// 获取激活的主题上下文
    pub async fn get_active_theme_context(&self) -> Result<Option<ThemeContext>> {
        if let Some(theme_name) = self.get_active_theme().await {
            Ok(Some(self.get_theme_context(&theme_name).await?))
        } else {
            Ok(None)
        }
    }
    
    /// 根据预览参数获取主题（用于主题预览功能）
    pub async fn get_theme_with_preview(
        &self,
        preview_theme: Option<&str>,
    ) -> Result<ThemeContext> {
        if let Some(preview) = preview_theme {
            // 预览模式：使用指定的主题
            self.get_theme_context(preview).await
        } else {
            // 正常模式：使用激活的主题
            self.get_active_theme_context().await?
                .ok_or_else(|| anyhow::anyhow!("No active theme"))
        }
    }
}

