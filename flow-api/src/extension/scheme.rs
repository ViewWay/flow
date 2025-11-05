use crate::extension::GroupVersionKind;

/// Scheme 定义扩展对象的类型和验证规则
#[derive(Debug, Clone)]
pub struct Scheme {
    pub gvk: GroupVersionKind,
    pub type_name: String,
    pub json_schema: Option<serde_json::Value>,
}

impl Scheme {
    pub fn new(
        gvk: GroupVersionKind,
        type_name: impl Into<String>,
    ) -> Self {
        Self {
            gvk,
            type_name: type_name.into(),
            json_schema: None,
        }
    }

    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.json_schema = Some(schema);
        self
    }
}

/// SchemeManager 管理所有扩展对象的Scheme定义
pub trait SchemeManager: Send + Sync {
    fn register(&mut self, scheme: Scheme) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn get(&self, gvk: &GroupVersionKind) -> Option<&Scheme>;
    fn remove(&mut self, gvk: &GroupVersionKind) -> Option<Scheme>;
    fn list(&self) -> Vec<&Scheme>;
}

/// DefaultSchemeManager 默认的Scheme管理器实现
pub struct DefaultSchemeManager {
    schemes: std::collections::HashMap<GroupVersionKind, Scheme>,
}

impl DefaultSchemeManager {
    pub fn new() -> Self {
        Self {
            schemes: std::collections::HashMap::new(),
        }
    }
}

impl Default for DefaultSchemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemeManager for DefaultSchemeManager {
    fn register(&mut self, scheme: Scheme) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.schemes.insert(scheme.gvk.clone(), scheme);
        Ok(())
    }

    fn get(&self, gvk: &GroupVersionKind) -> Option<&Scheme> {
        self.schemes.get(gvk)
    }

    fn remove(&mut self, gvk: &GroupVersionKind) -> Option<Scheme> {
        self.schemes.remove(gvk)
    }

    fn list(&self) -> Vec<&Scheme> {
        self.schemes.values().collect()
    }
}

