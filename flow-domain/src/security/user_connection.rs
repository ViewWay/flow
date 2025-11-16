use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// UserConnection实体的GVK常量
pub const USER_CONNECTION_GROUP: &str = "auth.halo.run";
pub const USER_CONNECTION_VERSION: &str = "v1alpha1";
pub const USER_CONNECTION_KIND: &str = "UserConnection";

/// UserConnection实体
/// 用于存储用户与OAuth2提供者的连接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConnection {
    pub metadata: Metadata,
    pub spec: UserConnectionSpec,
}

impl Extension for UserConnection {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(
            USER_CONNECTION_GROUP,
            USER_CONNECTION_VERSION,
            USER_CONNECTION_KIND,
        )
    }
}

/// UserConnectionSpec包含用户连接的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConnectionSpec {
    /// OAuth2提供者的注册ID（如：github、google等）
    pub registration_id: String,

    /// 关联的用户名（User的metadata.name）
    pub username: String,

    /// OAuth2提供者返回的用户唯一标识符
    /// 例如：GitHub的用户ID
    pub provider_user_id: String,

    /// 用户连接最后更新时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_api::extension::Metadata;

    #[test]
    fn test_user_connection_extension() {
        let connection = UserConnection {
            metadata: Metadata::new("test-connection"),
            spec: UserConnectionSpec {
                registration_id: "github".to_string(),
                username: "test-user".to_string(),
                provider_user_id: "12345".to_string(),
                updated_at: Some(Utc::now()),
            },
        };

        assert_eq!(connection.metadata().name, "test-connection");
        let gvk = connection.group_version_kind();
        assert_eq!(gvk.group, USER_CONNECTION_GROUP);
        assert_eq!(gvk.version, USER_CONNECTION_VERSION);
        assert_eq!(gvk.kind, USER_CONNECTION_KIND);
    }
}

