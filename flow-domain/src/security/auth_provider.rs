use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};

/// AuthProvider实体的GVK常量
pub const AUTH_PROVIDER_GROUP: &str = "auth.halo.run";
pub const AUTH_PROVIDER_VERSION: &str = "v1alpha1";
pub const AUTH_PROVIDER_KIND: &str = "AuthProvider";

/// AuthProvider实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProvider {
    pub metadata: Metadata,
    pub spec: AuthProviderSpec,
}

impl Extension for AuthProvider {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(AUTH_PROVIDER_GROUP, AUTH_PROVIDER_VERSION, AUTH_PROVIDER_KIND)
    }
}

/// AuthProviderSpec包含认证提供者的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProviderSpec {
    pub display_name: String,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub website: Option<String>,
    pub authentication_url: String,
    pub method: String, // "get" | "post"
    pub remember_me_support: Option<bool>,
    pub auth_type: String, // "form" | "oauth2" | "basic"
    /// ConfigMap引用（用于存储OAuth2配置等敏感信息）
    #[serde(rename = "configMapRef")]
    pub config_map_ref: Option<ConfigMapRef>,
}

/// ConfigMap引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapRef {
    /// ConfigMap名称
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_api::extension::Metadata;

    #[test]
    fn test_auth_provider_extension() {
        let provider = AuthProvider {
            metadata: Metadata::new("local"),
            spec: AuthProviderSpec {
                display_name: "Local".to_string(),
                description: None,
                logo: None,
                website: None,
                authentication_url: "/login".to_string(),
                method: "post".to_string(),
                remember_me_support: Some(true),
                auth_type: "form".to_string(),
                config_map_ref: None,
            },
        };

        assert_eq!(provider.metadata().name, "local");
        let gvk = provider.group_version_kind();
        assert_eq!(gvk.group, AUTH_PROVIDER_GROUP);
        assert_eq!(gvk.version, AUTH_PROVIDER_VERSION);
        assert_eq!(gvk.kind, AUTH_PROVIDER_KIND);
    }
}

