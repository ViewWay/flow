use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use crate::extension::ReactiveExtensionClient;

/// ConfigMap扩展对象（用于存储系统设置）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMap {
    pub metadata: Metadata,
    pub data: Option<HashMap<String, String>>,
}

impl Extension for ConfigMap {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new("", "v1alpha1", "ConfigMap")
    }
}

/// 系统设置常量
pub mod constants {
    pub const SYSTEM_CONFIG_MAP_NAME: &str = "system";
    pub const THEME_GROUP: &str = "theme";
}

/// 主题设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSetting {
    pub active: Option<String>,
}

/// 系统设置服务
#[async_trait]
pub trait SystemSettingService: Send + Sync {
    /// 获取主题设置
    async fn get_theme_setting(&self) -> Result<Option<ThemeSetting>>;
    
    /// 更新主题设置
    async fn update_theme_setting(&self, setting: ThemeSetting) -> Result<()>;
}

/// 默认系统设置服务实现
pub struct DefaultSystemSettingService {
    extension_client: Arc<ReactiveExtensionClient>,
}

impl DefaultSystemSettingService {
    pub fn new(extension_client: Arc<ReactiveExtensionClient>) -> Self {
        Self { extension_client }
    }
}

#[async_trait]
impl SystemSettingService for DefaultSystemSettingService {
    async fn get_theme_setting(&self) -> Result<Option<ThemeSetting>> {
        let config_map: Option<ConfigMap> = self.extension_client
            .fetch(constants::SYSTEM_CONFIG_MAP_NAME)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch config map: {}", e))?;
        
        if let Some(config_map) = config_map {
            if let Some(data) = config_map.data {
                if let Some(theme_json) = data.get(constants::THEME_GROUP) {
                    let setting: ThemeSetting = serde_json::from_str(theme_json)
                        .map_err(|e| anyhow::anyhow!("Failed to parse theme setting: {}", e))?;
                    return Ok(Some(setting));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn update_theme_setting(&self, setting: ThemeSetting) -> Result<()> {
        // 获取或创建ConfigMap
        let mut config_map: ConfigMap = self.extension_client
            .fetch(constants::SYSTEM_CONFIG_MAP_NAME)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch config map: {}", e))?
            .unwrap_or_else(|| {
                // 创建新的ConfigMap
                ConfigMap {
                    metadata: Metadata {
                        name: constants::SYSTEM_CONFIG_MAP_NAME.to_string(),
                        labels: None,
                        annotations: None,
                        creation_timestamp: None,
                        version: None,
                    },
                    data: Some(HashMap::new()),
                }
            });
        
        // 更新数据
        if config_map.data.is_none() {
            config_map.data = Some(HashMap::new());
        }
        
        let theme_json = serde_json::to_string(&setting)
            .map_err(|e| anyhow::anyhow!("Failed to serialize theme setting: {}", e))?;
        
        config_map.data.as_mut().unwrap()
            .insert(constants::THEME_GROUP.to_string(), theme_json);
        
        // 保存ConfigMap
        self.extension_client.update(config_map).await
            .map_err(|e| anyhow::anyhow!("Failed to update config map: {}", e))?;
        
        Ok(())
    }
}

