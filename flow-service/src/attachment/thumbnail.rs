use flow_domain::attachment::ThumbnailSize;
use std::path::{Path, PathBuf};
use anyhow::Result;

/// 缩略图服务trait
pub trait ThumbnailService: Send + Sync {
    /// 生成缩略图
    fn generate_thumbnail(&self, source_path: &Path, size: ThumbnailSize) -> Result<PathBuf>;
    
    /// 获取缩略图路径（如果存在）
    fn get_thumbnail_path(&self, source_path: &Path, size: ThumbnailSize) -> Option<PathBuf>;
    
    /// 检查是否为图片文件
    fn is_image(&self, media_type: &str) -> bool;
}

/// 默认缩略图服务实现
/// 使用image crate进行图片处理
pub struct DefaultThumbnailService {
    thumbnail_dir: PathBuf,
    quality: f32,
}

impl DefaultThumbnailService {
    pub fn new(thumbnail_dir: PathBuf, quality: f32) -> Self {
        Self {
            thumbnail_dir,
            quality: quality.max(0.0).min(1.0),
        }
    }
}

impl ThumbnailService for DefaultThumbnailService {
    fn generate_thumbnail(&self, source_path: &Path, size: ThumbnailSize) -> Result<PathBuf> {
        // TODO: 实现缩略图生成逻辑
        // 1. 检查源文件是否存在
        // 2. 检查缩略图是否已存在
        // 3. 使用image crate加载图片
        // 4. 调整图片大小
        // 5. 保存缩略图
        // 6. 返回缩略图路径
        
        // 临时实现：返回缩略图路径
        let thumbnail_path = self.get_thumbnail_path(source_path, size)
            .ok_or_else(|| anyhow::anyhow!("Failed to generate thumbnail path"))?;
        Ok(thumbnail_path)
    }
    
    fn get_thumbnail_path(&self, source_path: &Path, size: ThumbnailSize) -> Option<PathBuf> {
        // 生成缩略图路径：{thumbnail_dir}/{source_stem}_{size}.{ext}
        let stem = source_path.file_stem()?;
        let ext = source_path.extension()?;
        let filename = format!("{}_{}.{}", stem.to_string_lossy(), size.as_str(), ext.to_string_lossy());
        Some(self.thumbnail_dir.join(filename))
    }
    
    fn is_image(&self, media_type: &str) -> bool {
        media_type.starts_with("image/")
    }
}

