use flow_api::extension::Extension;
use crate::database::extension_store::Model as ExtensionStoreModel;
use serde::{Serialize, Deserialize};

/// ExtensionConverter 负责Extension和ExtensionStore之间的转换
pub trait ExtensionConverter: Send + Sync {
    fn convert_to<E: Extension + Serialize>(&self, extension: &E) -> Result<ExtensionStoreModel, Box<dyn std::error::Error + Send + Sync>>;
    fn convert_from<E: Extension + for<'de> Deserialize<'de>>(&self, store: &ExtensionStoreModel) -> Result<E, Box<dyn std::error::Error + Send + Sync>>;
}

/// JSONExtensionConverter 使用JSON序列化的转换器
pub struct JSONExtensionConverter;

impl ExtensionConverter for JSONExtensionConverter {
    fn convert_to<E: Extension + Serialize>(&self, extension: &E) -> Result<ExtensionStoreModel, Box<dyn std::error::Error + Send + Sync>> {
        let gvk = extension.group_version_kind();
        let metadata = extension.metadata();
        
        // 构建存储名称: {gvk}/{name}
        let store_name = format!("{}/{}/{}", gvk.group, gvk.version, metadata.name);
        
        // 序列化扩展对象
        let data = serde_json::to_vec(extension)?;
        
        Ok(ExtensionStoreModel {
            name: store_name,
            data,
            version: metadata.version.map(|v| v as i64),
        })
    }

    fn convert_from<E: Extension + for<'de> Deserialize<'de>>(&self, store: &ExtensionStoreModel) -> Result<E, Box<dyn std::error::Error + Send + Sync>> {
        // 反序列化扩展对象
        let extension: E = serde_json::from_slice(&store.data)?;
        Ok(extension)
    }
}

