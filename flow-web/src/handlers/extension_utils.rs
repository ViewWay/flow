use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// DynamicExtension 是一个通用的Extension包装类型
/// 用于处理动态的Extension对象（通过JSON）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicExtension {
    pub metadata: Metadata,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    #[serde(flatten)]
    pub data: Value,
}

impl DynamicExtension {
    /// 从JSON Value创建DynamicExtension
    pub fn from_value(mut value: Value) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 提取metadata
        let metadata = value
            .get("metadata")
            .and_then(|m| serde_json::from_value(m.clone()).ok())
            .ok_or_else(|| "Missing metadata".to_string())?;

        // 提取apiVersion和kind
        let api_version = value
            .get("apiVersion")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing apiVersion".to_string())?
            .to_string();

        let kind = value
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing kind".to_string())?
            .to_string();

        // 移除metadata、apiVersion、kind，剩余部分作为data
        value.as_object_mut().map(|obj| {
            obj.remove("metadata");
            obj.remove("apiVersion");
            obj.remove("kind");
        });

        Ok(Self {
            metadata,
            api_version,
            kind,
            data: value,
        })
    }

    /// 转换为JSON Value
    pub fn to_value(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("metadata".to_string(), serde_json::to_value(&self.metadata).unwrap());
        obj.insert("apiVersion".to_string(), serde_json::to_value(&self.api_version).unwrap());
        obj.insert("kind".to_string(), serde_json::to_value(&self.kind).unwrap());
        
        // 合并data中的字段
        if let Some(data_obj) = self.data.as_object() {
            for (k, v) in data_obj {
                obj.insert(k.clone(), v.clone());
            }
        }
        
        Value::Object(obj)
    }

    /// 解析GVK（从apiVersion中提取group和version）
    fn parse_gvk(&self) -> GroupVersionKind {
        // apiVersion格式: group/version 或 version
        let parts: Vec<&str> = self.api_version.split('/').collect();
        if parts.len() == 2 {
            GroupVersionKind::new(parts[0], parts[1], &self.kind)
        } else {
            // 如果没有group，使用空字符串
            GroupVersionKind::new("", parts[0], &self.kind)
        }
    }
}

impl Extension for DynamicExtension {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        self.parse_gvk()
    }
}

