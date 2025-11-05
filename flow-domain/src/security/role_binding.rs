use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};

/// RoleBinding实体的GVK常量
pub const ROLE_BINDING_GROUP: &str = "";
pub const ROLE_BINDING_VERSION: &str = "v1alpha1";
pub const ROLE_BINDING_KIND: &str = "RoleBinding";

/// User和Role的GVK常量（用于Subject和RoleRef）
pub const USER_KIND: &str = "User";
pub const ROLE_KIND: &str = "Role";

/// RoleBinding实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBinding {
    pub metadata: Metadata,
    pub subjects: Vec<Subject>,
    pub role_ref: RoleRef,
}

impl Extension for RoleBinding {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(ROLE_BINDING_GROUP, ROLE_BINDING_VERSION, ROLE_BINDING_KIND)
    }
}

impl RoleBinding {
    /// 创建一个新的RoleBinding
    pub fn create(username: &str, role_name: &str) -> Self {
        let binding_name = format!("{}-{}-binding", username, role_name);
        
        let role_ref = RoleRef {
            kind: ROLE_KIND.to_string(),
            name: role_name.to_string(),
            api_group: "".to_string(),
        };

        let subject = Subject {
            kind: USER_KIND.to_string(),
            name: username.to_string(),
            api_group: "".to_string(),
        };

        Self {
            metadata: Metadata::new(binding_name),
            subjects: vec![subject],
            role_ref,
        }
    }

    /// 检查RoleBinding是否包含指定的用户
    pub fn contains_user(&self, username: &str) -> bool {
        self.subjects.iter().any(|s| {
            s.kind == USER_KIND && s.name == username
        })
    }
}

/// Subject表示RoleBinding的绑定对象
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subject {
    /// Kind是资源的类型
    pub kind: String,
    /// Name是资源名称
    pub name: String,
    /// ApiGroup是资源的API组
    pub api_group: String,
}

impl Subject {
    /// 将Subject转换为字符串表示（格式：kind/name）
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.kind, self.name)
    }

    /// 检查Subject是否是用户类型
    pub fn is_user(&self) -> bool {
        self.kind == USER_KIND
    }
}

/// RoleRef包含指向要使用的角色的信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleRef {
    /// Kind是被引用的资源类型
    pub kind: String,
    /// Name是被引用的资源名称
    pub name: String,
    /// ApiGroup是被引用的资源的组
    pub api_group: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_api::extension::Metadata;

    #[test]
    fn test_role_binding_extension() {
        let binding = RoleBinding::create("test-user", "admin-role");
        
        assert_eq!(binding.metadata().name, "test-user-admin-role-binding");
        let gvk = binding.group_version_kind();
        assert_eq!(gvk.group, ROLE_BINDING_GROUP);
        assert_eq!(gvk.version, ROLE_BINDING_VERSION);
        assert_eq!(gvk.kind, ROLE_BINDING_KIND);
    }

    #[test]
    fn test_role_binding_contains_user() {
        let binding = RoleBinding::create("test-user", "admin-role");
        assert!(binding.contains_user("test-user"));
        assert!(!binding.contains_user("other-user"));
    }

    #[test]
    fn test_subject_to_string() {
        let subject = Subject {
            kind: USER_KIND.to_string(),
            name: "test-user".to_string(),
            api_group: "".to_string(),
        };
        assert_eq!(subject.to_string(), "User/test-user");
        assert!(subject.is_user());
    }
}

