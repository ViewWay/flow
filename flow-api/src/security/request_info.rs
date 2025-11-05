/// 从HTTP请求解析的资源信息
/// 用于授权决策
#[derive(Debug, Clone)]
pub struct RequestInfo {
    /// 是否是资源请求（而非非资源URL）
    pub is_resource_request: bool,
    /// 请求路径
    pub path: String,
    /// HTTP方法（转换为小写）
    pub verb: String,
    /// API组（如果有）
    pub api_group: Option<String>,
    /// API版本（如果有）
    pub api_version: Option<String>,
    /// 资源名称（如果有）
    pub resource: Option<String>,
    /// 资源实例名称（如果有）
    pub name: Option<String>,
    /// 子资源名称（如果有）
    pub subresource: Option<String>,
    /// 用户空间（如果有）
    pub userspace: Option<String>,
}

impl RequestInfo {
    /// 从HTTP请求信息解析RequestInfo
    pub fn from_request(method: &str, path: &str) -> Self {
        let verb = method.to_lowercase();

        // 解析路径
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // 检查是否是资源请求
        let is_resource_request = Self::is_resource_path(&path);

        if is_resource_request {
            Self::parse_resource_request(&path, &verb, &parts)
        } else {
        Self {
            is_resource_request: false,
            path: path.to_string(),
            verb,
            api_group: None,
            api_version: None,
            resource: None,
            name: None,
            subresource: None,
            userspace: None,
        }
        }
    }

    /// 解析资源请求
    fn parse_resource_request(path: &str, verb: &str, parts: &[&str]) -> Self {
        // 解析格式：/api/v1alpha1/posts 或 /api/v1alpha1/posts/{name}
        // 或 /apis/{group}/{version}/{resource} 或 /api/v1alpha1/uc/posts
        
        let mut api_group = None;
        let mut api_version = None;
        let mut resource = None;
        let mut name = None;
        let mut subresource = None;
        let mut userspace = None;

        if path.starts_with("/api/") {
            // 标准API路径：/api/v1alpha1/posts
            if parts.len() >= 2 {
                api_version = Some(parts[1].to_string());
                api_group = Some("".to_string()); // 空字符串表示核心API组

                // 检查是否是UC（用户中心）路径
                if parts.len() >= 3 && parts[2] == "uc" {
                    userspace = Some(parts[2].to_string());
                    if parts.len() >= 4 {
                        resource = Some(parts[3].to_string());
                        if parts.len() >= 5 {
                            name = Some(parts[4].to_string());
                            if parts.len() >= 6 {
                                subresource = Some(parts[5].to_string());
                            }
                        }
                    }
                } else if parts.len() >= 3 {
                    resource = Some(parts[2].to_string());
                    if parts.len() >= 4 {
                        name = Some(parts[3].to_string());
                        if parts.len() >= 5 {
                            subresource = Some(parts[4].to_string());
                        }
                    }
                }
            }
        } else if path.starts_with("/apis/") {
            // Extension API路径：/apis/{group}/{version}/{resource}
            if parts.len() >= 3 {
                api_group = Some(parts[1].to_string());
                api_version = Some(parts[2].to_string());
                if parts.len() >= 4 {
                    resource = Some(parts[3].to_string());
                    if parts.len() >= 5 {
                        name = Some(parts[4].to_string());
                        if parts.len() >= 6 {
                            subresource = Some(parts[5].to_string());
                        }
                    }
                }
            }
        }

        Self {
            is_resource_request: true,
            path: path.to_string(),
            verb: verb.to_string(),
            api_group,
            api_version,
            resource,
            name,
            subresource,
            userspace,
        }
    }

    /// 检查路径是否是资源请求路径
    fn is_resource_path(path: &str) -> bool {
        path.starts_with("/api/") || path.starts_with("/apis/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_api_path() {
        let info = RequestInfo::from_request("GET", "/api/v1alpha1/posts");

        assert!(info.is_resource_request);
        assert_eq!(info.verb, "get");
        assert_eq!(info.api_group, Some("".to_string()));
        assert_eq!(info.api_version, Some("v1alpha1".to_string()));
        assert_eq!(info.resource, Some("posts".to_string()));
        assert_eq!(info.name, None);
    }

    #[test]
    fn test_parse_api_path_with_name() {
        let info = RequestInfo::from_request("GET", "/api/v1alpha1/posts/my-post");

        assert!(info.is_resource_request);
        assert_eq!(info.resource, Some("posts".to_string()));
        assert_eq!(info.name, Some("my-post".to_string()));
    }

    #[test]
    fn test_parse_extension_api_path() {
        let info = RequestInfo::from_request("GET", "/apis/content.halo.run/v1alpha1/posts");

        assert!(info.is_resource_request);
        assert_eq!(info.api_group, Some("content.halo.run".to_string()));
        assert_eq!(info.api_version, Some("v1alpha1".to_string()));
        assert_eq!(info.resource, Some("posts".to_string()));
    }

    #[test]
    fn test_parse_uc_path() {
        let info = RequestInfo::from_request("GET", "/api/v1alpha1/uc/posts");

        assert!(info.is_resource_request);
        assert_eq!(info.userspace, Some("uc".to_string()));
        assert_eq!(info.resource, Some("posts".to_string()));
    }

    #[test]
    fn test_parse_non_resource_path() {
        let info = RequestInfo::from_request("POST", "/login");

        assert!(!info.is_resource_request);
        assert_eq!(info.verb, "post");
        assert_eq!(info.path, "/login");
    }
}

