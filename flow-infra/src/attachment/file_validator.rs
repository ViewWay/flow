use std::collections::HashSet;

/// 文件类型验证器
pub struct FileTypeValidator {
    /// 允许的MIME类型集合
    allowed_types: HashSet<String>,
    /// 允许的文件扩展名集合
    allowed_extensions: HashSet<String>,
}

impl FileTypeValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self {
            allowed_types: HashSet::new(),
            allowed_extensions: HashSet::new(),
        }
    }
    
    /// 添加允许的MIME类型
    pub fn allow_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.allowed_types.insert(mime_type.into());
        self
    }
    
    /// 添加允许的文件扩展名
    pub fn allow_extension(mut self, ext: impl Into<String>) -> Self {
        self.allowed_extensions.insert(ext.into().to_lowercase());
        self
    }
    
    /// 添加常见的图片类型
    pub fn allow_images(mut self) -> Self {
        self.allowed_types.insert("image/jpeg".to_string());
        self.allowed_types.insert("image/png".to_string());
        self.allowed_types.insert("image/gif".to_string());
        self.allowed_types.insert("image/webp".to_string());
        self.allowed_types.insert("image/svg+xml".to_string());
        self.allowed_extensions.insert("jpg".to_string());
        self.allowed_extensions.insert("jpeg".to_string());
        self.allowed_extensions.insert("png".to_string());
        self.allowed_extensions.insert("gif".to_string());
        self.allowed_extensions.insert("webp".to_string());
        self.allowed_extensions.insert("svg".to_string());
        self
    }
    
    /// 添加常见的文档类型
    pub fn allow_documents(mut self) -> Self {
        self.allowed_types.insert("application/pdf".to_string());
        self.allowed_types.insert("application/msword".to_string());
        self.allowed_types.insert("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string());
        self.allowed_types.insert("application/vnd.ms-excel".to_string());
        self.allowed_types.insert("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string());
        self.allowed_extensions.insert("pdf".to_string());
        self.allowed_extensions.insert("doc".to_string());
        self.allowed_extensions.insert("docx".to_string());
        self.allowed_extensions.insert("xls".to_string());
        self.allowed_extensions.insert("xlsx".to_string());
        self
    }
    
    /// 添加常见的视频类型
    pub fn allow_videos(mut self) -> Self {
        self.allowed_types.insert("video/mp4".to_string());
        self.allowed_types.insert("video/webm".to_string());
        self.allowed_types.insert("video/ogg".to_string());
        self.allowed_extensions.insert("mp4".to_string());
        self.allowed_extensions.insert("webm".to_string());
        self.allowed_extensions.insert("ogg".to_string());
        self
    }
    
    /// 添加常见的音频类型
    pub fn allow_audio(mut self) -> Self {
        self.allowed_types.insert("audio/mpeg".to_string());
        self.allowed_types.insert("audio/wav".to_string());
        self.allowed_types.insert("audio/ogg".to_string());
        self.allowed_extensions.insert("mp3".to_string());
        self.allowed_extensions.insert("wav".to_string());
        self.allowed_extensions.insert("ogg".to_string());
        self
    }
    
    /// 允许所有类型（默认）
    pub fn allow_all(mut self) -> Self {
        // 清空限制，表示允许所有类型
        self.allowed_types.clear();
        self.allowed_extensions.clear();
        self
    }
    
    /// 验证文件类型
    pub fn validate(&self, mime_type: Option<&str>, filename: &str) -> bool {
        // 如果允许所有类型，直接返回true
        if self.allowed_types.is_empty() && self.allowed_extensions.is_empty() {
            return true;
        }
        
        // 检查MIME类型
        if let Some(mime) = mime_type {
            if self.allowed_types.contains(mime) {
                return true;
            }
        }
        
        // 检查文件扩展名
        if let Some(ext) = filename.split('.').last() {
            if self.allowed_extensions.contains(&ext.to_lowercase()) {
                return true;
            }
        }
        
        false
    }
}

impl Default for FileTypeValidator {
    fn default() -> Self {
        Self::new().allow_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_type_validator() {
        let validator = FileTypeValidator::new()
            .allow_images();
        
        assert!(validator.validate(Some("image/jpeg"), "test.jpg"));
        assert!(validator.validate(Some("image/png"), "test.png"));
        assert!(!validator.validate(Some("application/pdf"), "test.pdf"));
        
        let validator = FileTypeValidator::new()
            .allow_images()
            .allow_documents();
        
        assert!(validator.validate(Some("image/jpeg"), "test.jpg"));
        assert!(validator.validate(Some("application/pdf"), "test.pdf"));
    }
}

