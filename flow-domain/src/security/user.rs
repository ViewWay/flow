use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// User实体的GVK常量
pub const USER_GROUP: &str = "";
pub const USER_VERSION: &str = "v1alpha1";
pub const USER_KIND: &str = "User";

/// User相关的索引和注解常量
pub const USER_RELATED_ROLES_INDEX: &str = "roles";
pub const ROLE_NAMES_ANNO: &str = "rbac.authorization.halo.run/role-names";
pub const EMAIL_TO_VERIFY: &str = "halo.run/email-to-verify";
pub const LAST_AVATAR_ATTACHMENT_NAME_ANNO: &str = "halo.run/last-avatar-attachment-name";
pub const AVATAR_ATTACHMENT_NAME_ANNO: &str = "halo.run/avatar-attachment-name";
pub const HIDDEN_USER_LABEL: &str = "halo.run/hidden-user";
pub const REQUEST_TO_UPDATE: &str = "halo.run/request-to-update";

/// User实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub metadata: Metadata,
    pub spec: UserSpec,
    pub status: Option<UserStatus>,
}

impl Extension for User {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(USER_GROUP, USER_VERSION, USER_KIND)
    }
}

/// UserSpec包含用户的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSpec {
    pub display_name: String,
    pub avatar: Option<String>,
    pub email: String,
    pub email_verified: Option<bool>,
    pub phone: Option<String>,
    pub password: Option<String>, // 加密后的密码
    pub bio: Option<String>,
    pub registered_at: Option<DateTime<Utc>>,
    pub two_factor_auth_enabled: Option<bool>,
    pub totp_encrypted_secret: Option<String>,
    pub disabled: Option<bool>,
    pub login_history_limit: Option<u32>,
}

impl Default for UserSpec {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            avatar: None,
            email: String::new(),
            email_verified: Some(false),
            phone: None,
            password: None,
            bio: None,
            registered_at: Some(Utc::now()),
            two_factor_auth_enabled: Some(false),
            totp_encrypted_secret: None,
            disabled: Some(false),
            login_history_limit: Some(10),
        }
    }
}

/// UserStatus包含用户的状态信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStatus {
    pub permalink: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_api::extension::Metadata;

    #[test]
    fn test_user_extension() {
        let user = User {
            metadata: Metadata::new("test-user"),
            spec: UserSpec {
                display_name: "Test User".to_string(),
                email: "test@example.com".to_string(),
                ..Default::default()
            },
            status: None,
        };

        assert_eq!(user.metadata().name, "test-user");
        let gvk = user.group_version_kind();
        assert_eq!(gvk.group, USER_GROUP);
        assert_eq!(gvk.version, USER_VERSION);
        assert_eq!(gvk.kind, USER_KIND);
    }
}

