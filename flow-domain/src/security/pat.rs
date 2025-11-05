use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// PersonalAccessToken实体的GVK常量
pub const PAT_GROUP: &str = "";
pub const PAT_VERSION: &str = "v1alpha1";
pub const PAT_KIND: &str = "PersonalAccessToken";

/// PersonalAccessToken实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccessToken {
    pub metadata: Metadata,
    pub spec: PatSpec,
}

impl Extension for PersonalAccessToken {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(PAT_GROUP, PAT_VERSION, PAT_KIND)
    }
}

/// PatSpec包含PAT的规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatSpec {
    /// Token ID，用于验证JWT中的jti claim
    pub token_id: String,
    /// PAT绑定的角色列表
    pub roles: Vec<String>,
    /// 最后使用时间
    pub last_used: Option<DateTime<Utc>>,
    /// 是否已撤销
    pub revoked: bool,
    /// 过期时间（可选）
    pub expires_at: Option<DateTime<Utc>>,
    /// 描述信息
    pub description: Option<String>,
}

impl Default for PatSpec {
    fn default() -> Self {
        Self {
            token_id: String::new(),
            roles: Vec::new(),
            last_used: None,
            revoked: false,
            expires_at: None,
            description: None,
        }
    }
}

impl PersonalAccessToken {
    /// 检查PAT是否有效（未撤销且未过期）
    pub fn is_valid(&self) -> bool {
        if self.spec.revoked {
            return false;
        }

        if let Some(expires_at) = self.spec.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }

        true
    }

    /// 检查Token ID是否匹配
    pub fn matches_token_id(&self, token_id: &str) -> bool {
        self.spec.token_id == token_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_api::extension::Metadata;

    #[test]
    fn test_pat_extension() {
        let pat = PersonalAccessToken {
            metadata: Metadata::new("test-pat"),
            spec: PatSpec {
                token_id: "test-token-id".to_string(),
                roles: vec!["admin".to_string()],
                ..Default::default()
            },
        };

        assert_eq!(pat.metadata().name, "test-pat");
        let gvk = pat.group_version_kind();
        assert_eq!(gvk.group, PAT_GROUP);
        assert_eq!(gvk.version, PAT_VERSION);
        assert_eq!(gvk.kind, PAT_KIND);
    }

    #[test]
    fn test_pat_is_valid() {
        let mut pat = PersonalAccessToken {
            metadata: Metadata::new("test-pat"),
            spec: PatSpec {
                token_id: "test-token-id".to_string(),
                roles: Vec::new(),
                ..Default::default()
            },
        };

        // 未撤销的PAT应该有效
        assert!(pat.is_valid());

        // 撤销的PAT应该无效
        pat.spec.revoked = true;
        assert!(!pat.is_valid());

        // 恢复后应该有效
        pat.spec.revoked = false;
        assert!(pat.is_valid());

        // 过期的PAT应该无效
        pat.spec.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!pat.is_valid());
    }

    #[test]
    fn test_pat_matches_token_id() {
        let pat = PersonalAccessToken {
            metadata: Metadata::new("test-pat"),
            spec: PatSpec {
                token_id: "test-token-id".to_string(),
                roles: Vec::new(),
                ..Default::default()
            },
        };

        assert!(pat.matches_token_id("test-token-id"));
        assert!(!pat.matches_token_id("other-token-id"));
    }
}

