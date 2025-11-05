use flow_api::extension::{Extension, GroupVersionKind, Metadata};
use serde::{Deserialize, Serialize};

/// Role实体的GVK常量
pub const ROLE_GROUP: &str = "";
pub const ROLE_VERSION: &str = "v1alpha1";
pub const ROLE_KIND: &str = "Role";

/// Role相关的注解常量
pub const ROLE_DEPENDENCY_RULES: &str = "rbac.authorization.halo.run/dependency-rules";
pub const ROLE_DEPENDENCIES_ANNO: &str = "rbac.authorization.halo.run/dependencies";
pub const UI_PERMISSIONS_ANNO: &str = "rbac.authorization.halo.run/ui-permissions";
pub const UI_PERMISSIONS_AGGREGATED_ANNO: &str = "rbac.authorization.halo.run/ui-permissions-aggregated";
pub const SYSTEM_RESERVED_LABELS: &str = "rbac.authorization.halo.run/system-reserved";
pub const HIDDEN_LABEL_NAME: &str = "halo.run/hidden";
pub const TEMPLATE_LABEL_NAME: &str = "halo.run/role-template";
pub const ROLE_AGGREGATE_LABEL_PREFIX: &str = "rbac.authorization.halo.run/aggregate-to-";

/// Role实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub metadata: Metadata,
    pub rules: Vec<PolicyRule>,
}

impl Extension for Role {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn group_version_kind(&self) -> GroupVersionKind {
        GroupVersionKind::new(ROLE_GROUP, ROLE_VERSION, ROLE_KIND)
    }
}

/// PolicyRule定义授权策略规则
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRule {
    /// APIGroups是包含资源的API组名称
    /// 如果指定了多个API组，对任何API组中枚举的资源执行的任何操作都将被允许
    #[serde(default)]
    pub api_groups: Vec<String>,

    /// Resources是此规则适用的资源列表
    /// '*'表示指定apiGroups中的所有资源
    /// '*\/foo'表示指定apiGroups中所有资源的子资源'foo'
    #[serde(default)]
    pub resources: Vec<String>,

    /// ResourceNames是规则适用的可选白名单名称
    /// 空集合意味着允许所有内容
    #[serde(default)]
    pub resource_names: Vec<String>,

    /// NonResourceURLs是用户应该有权访问的部分URL集合
    /// 允许使用*，但只能作为路径中的完整、最后一步
    #[serde(default)]
    pub non_resource_urls: Vec<String>,

    /// Verbs是允许的操作列表
    #[serde(default)]
    pub verbs: Vec<String>,
}

impl Default for PolicyRule {
    fn default() -> Self {
        Self {
            api_groups: Vec::new(),
            resources: Vec::new(),
            resource_names: Vec::new(),
            non_resource_urls: Vec::new(),
            verbs: Vec::new(),
        }
    }
}

impl PolicyRule {
    /// 检查此规则是否允许给定的请求
    /// 注意：这里接受RequestInfo参数，但在flow-domain中不直接依赖flow-api::security
    /// 实际使用时会在flow-service层进行匹配
    pub fn matches_request(
        &self,
        verb: &str,
        api_group: Option<&str>,
        resource: Option<&str>,
        name: Option<&str>,
        subresource: Option<&str>,
    ) -> bool {
        // 检查verb是否匹配
        if !self.verbs.is_empty() && !self.verbs.contains(&verb.to_string()) {
            return false;
        }

        // 检查api_group是否匹配
        if let Some(api_group) = api_group {
            if !self.api_groups.is_empty() && !self.matches_api_group(api_group) {
                return false;
            }
        }

        // 检查resource是否匹配
        if let Some(resource) = resource {
            if !self.resources.is_empty() && !self.matches_resource(resource) {
                return false;
            }
        }

        // 检查resource_name是否匹配
        if let Some(name) = name {
            if !self.resource_names.is_empty() && !self.resource_names.contains(&name.to_string()) {
                return false;
            }
        }

        // 检查subresource是否匹配
        if let Some(subresource) = subresource {
            // 如果resources中包含 "resource/subresource" 格式，需要检查
            let resource_with_sub = format!("{}/{}", 
                resource.unwrap_or(""), 
                subresource);
            if !self.resources.is_empty() && 
               !self.matches_resource(&resource_with_sub) &&
               !self.matches_resource(&format!("*/{}", subresource)) {
                return false;
            }
        }

        true
    }

    /// 检查api_group是否匹配（支持通配符）
    fn matches_api_group(&self, api_group: &str) -> bool {
        self.api_groups.iter().any(|pattern| {
            Self::matches_wildcard(pattern, api_group)
        })
    }

    /// 检查resource是否匹配（支持通配符）
    fn matches_resource(&self, resource: &str) -> bool {
        self.resources.iter().any(|pattern| {
            Self::matches_wildcard(pattern, resource)
        })
    }

    /// 检查non_resource_url是否匹配（支持通配符）
    fn matches_non_resource_url(&self, url: &str) -> bool {
        self.non_resource_urls.iter().any(|pattern| {
            Self::matches_wildcard(pattern, url)
        })
    }

    /// 通配符匹配函数
    /// 支持 '*' 匹配任意字符序列
    /// 支持 '*/subresource' 匹配任意资源的子资源
    fn matches_wildcard(pattern: &str, value: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // 处理 "*/subresource" 格式
        if let Some(subresource) = pattern.strip_prefix("*/") {
            return value.ends_with(&format!("/{}", subresource));
        }

        // 处理 "*" 在中间的情况（如 "posts/*"）
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return value.starts_with(prefix) && value.ends_with(suffix);
            }
        }

        // 精确匹配
        pattern == value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_api::extension::Metadata;

    #[test]
    fn test_role_extension() {
        let role = Role {
            metadata: Metadata::new("test-role"),
            rules: Vec::new(),
        };

        assert_eq!(role.metadata().name, "test-role");
        let gvk = role.group_version_kind();
        assert_eq!(gvk.group, ROLE_GROUP);
        assert_eq!(gvk.version, ROLE_VERSION);
        assert_eq!(gvk.kind, ROLE_KIND);
    }

    #[test]
    fn test_policy_rule_wildcard_matching() {
        // 测试 '*' 匹配
        assert!(PolicyRule::matches_wildcard("*", "anything"));
        assert!(PolicyRule::matches_wildcard("*", ""));

        // 测试精确匹配
        assert!(PolicyRule::matches_wildcard("posts", "posts"));
        assert!(!PolicyRule::matches_wildcard("posts", "post"));

        // 测试 "*/subresource" 格式
        assert!(PolicyRule::matches_wildcard("*/comments", "posts/comments"));
        assert!(PolicyRule::matches_wildcard("*/comments", "pages/comments"));
        assert!(!PolicyRule::matches_wildcard("*/comments", "posts"));

        // 测试 "prefix*suffix" 格式
        assert!(PolicyRule::matches_wildcard("post*", "posts"));
        assert!(PolicyRule::matches_wildcard("post*", "post"));
    }

    #[test]
    fn test_policy_rule_matches_request() {
        let rule = PolicyRule {
            api_groups: vec!["".to_string(), "content.halo.run".to_string()],
            resources: vec!["posts".to_string()],
            resource_names: Vec::new(),
            non_resource_urls: Vec::new(),
            verbs: vec!["get".to_string(), "list".to_string()],
        };

        assert!(rule.matches_request("get", Some(""), Some("posts"), None, None));
        assert!(!rule.matches_request("post", Some(""), Some("posts"), None, None));
    }
}

