pub mod storage;
pub mod resource_mapping;

pub use storage::LocalAttachmentStorage;
pub use resource_mapping::ResourceMapping;

/// 附件存储trait
pub trait AttachmentStorage: Send + Sync {
    /// 保存文件
    fn save(&self, content: &[u8], path: &std::path::Path) -> anyhow::Result<()>;
    
    /// 读取文件
    fn read(&self, path: &std::path::Path) -> anyhow::Result<Vec<u8>>;
    
    /// 删除文件
    fn delete(&self, path: &std::path::Path) -> anyhow::Result<()>;
    
    /// 检查文件是否存在
    fn exists(&self, path: &std::path::Path) -> bool;
    
    /// 获取文件大小
    fn size(&self, path: &std::path::Path) -> anyhow::Result<u64>;
}

