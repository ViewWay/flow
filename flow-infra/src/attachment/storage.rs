use super::AttachmentStorage;
use std::path::{Path, PathBuf};
use tokio::fs;
use anyhow::Result;

/// 本地文件存储实现
pub struct LocalAttachmentStorage {
    base_path: PathBuf,
}

impl LocalAttachmentStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
    
    /// 构建完整文件路径
    fn build_path(&self, relative_path: &Path) -> PathBuf {
        self.base_path.join(relative_path)
    }
}

impl AttachmentStorage for LocalAttachmentStorage {
    fn save(&self, content: &[u8], path: &Path) -> Result<()> {
        let full_path = self.build_path(path);
        
        // 创建父目录
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 保存文件
        std::fs::write(&full_path, content)?;
        Ok(())
    }
    
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let full_path = self.build_path(path);
        let content = std::fs::read(&full_path)?;
        Ok(content)
    }
    
    fn delete(&self, path: &Path) -> Result<()> {
        let full_path = self.build_path(path);
        if full_path.exists() {
            std::fs::remove_file(&full_path)?;
        }
        Ok(())
    }
    
    fn exists(&self, path: &Path) -> bool {
        let full_path = self.build_path(path);
        full_path.exists()
    }
    
    fn size(&self, path: &Path) -> Result<u64> {
        let full_path = self.build_path(path);
        let metadata = std::fs::metadata(&full_path)?;
        Ok(metadata.len())
    }
}

