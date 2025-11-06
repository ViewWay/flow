use flow_domain::attachment::ThumbnailSize;
use std::path::{Path, PathBuf};
use anyhow::Result;
use image::{ImageReader, DynamicImage, imageops::FilterType};
use std::fs;

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
        // 1. 检查源文件是否存在
        if !source_path.exists() {
            anyhow::bail!("Source file does not exist: {}", source_path.display());
        }
        
        // 2. 获取缩略图路径
        let thumbnail_path = self.get_thumbnail_path(source_path, size)
            .ok_or_else(|| anyhow::anyhow!("Failed to generate thumbnail path"))?;
        
        // 3. 检查缩略图是否已存在
        if thumbnail_path.exists() {
            return Ok(thumbnail_path);
        }
        
        // 4. 创建缩略图目录
        if let Some(parent) = thumbnail_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 5. 使用image crate加载图片
        let img = ImageReader::open(source_path)?
            .decode()
            .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
        
        // 6. 调整图片大小
        let width = size.width();
        let resized = if img.width() > width {
            img.resize(width, (img.height() as f32 * (width as f32 / img.width() as f32)) as u32, FilterType::Lanczos3)
        } else {
            // 如果原图小于目标尺寸，保持原图大小
            img
        };
        
        // 7. 保存缩略图
        resized.save_with_format(&thumbnail_path, image::ImageFormat::Jpeg)
            .map_err(|e| anyhow::anyhow!("Failed to save thumbnail: {}", e))?;
        
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

