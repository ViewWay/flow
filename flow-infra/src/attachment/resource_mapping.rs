use regex::Regex;
use std::path::PathBuf;

/// 资源映射配置
/// 用于将URL路径映射到本地文件系统路径
#[derive(Debug, Clone)]
pub struct ResourceMapping {
    /// 路径模式（如：/upload/**）
    pub path_pattern: String,
    
    /// 本地文件系统路径列表（相对于附件根目录）
    pub locations: Vec<String>,
    
    /// 编译后的正则表达式
    pattern_regex: Option<Regex>,
}

impl ResourceMapping {
    pub fn new(path_pattern: String, locations: Vec<String>) -> Self {
        Self {
            path_pattern,
            locations,
            pattern_regex: None,
        }
    }
    
    /// 编译路径模式为正则表达式
    pub fn compile(&mut self) -> Result<(), regex::Error> {
        // 将路径模式转换为正则表达式
        // 例如：/upload/** -> ^/upload/.*
        let pattern = self.path_pattern
            .replace("**", ".*")
            .replace("*", "[^/]*");
        let regex_pattern = format!("^{}", pattern);
        self.pattern_regex = Some(Regex::new(&regex_pattern)?);
        Ok(())
    }
    
    /// 检查路径是否匹配
    pub fn matches(&self, path: &str) -> bool {
        if let Some(ref regex) = self.pattern_regex {
            regex.is_match(path)
        } else {
            // 如果未编译，使用简单的字符串匹配
            path.starts_with(&self.path_pattern.trim_end_matches("/**"))
        }
    }
    
    /// 解析路径到本地文件系统路径
    /// 返回第一个匹配的location
    pub fn resolve_path(&self, url_path: &str) -> Option<PathBuf> {
        if !self.matches(url_path) {
            return None;
        }
        
        // 提取路径中的相对部分
        let relative_path = if let Some(prefix) = self.path_pattern.strip_suffix("/**") {
            url_path.strip_prefix(prefix)?
        } else if let Some(prefix) = self.path_pattern.strip_suffix("*") {
            url_path.strip_prefix(prefix)?
        } else {
            url_path.strip_prefix(&self.path_pattern)?
        };
        
        // 使用第一个location
        if let Some(location) = self.locations.first() {
            Some(PathBuf::from(location).join(relative_path.trim_start_matches('/')))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_mapping() {
        let mut mapping = ResourceMapping::new(
            "/upload/**".to_string(),
            vec!["attachments".to_string()],
        );
        mapping.compile().unwrap();
        
        assert!(mapping.matches("/upload/image.jpg"));
        assert!(mapping.matches("/upload/subdir/image.jpg"));
        assert!(!mapping.matches("/other/image.jpg"));
        
        let path = mapping.resolve_path("/upload/image.jpg");
        assert_eq!(path, Some(PathBuf::from("attachments/image.jpg")));
    }
}

