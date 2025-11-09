use flow_domain::theme::Theme;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use zip::ZipArchive;
use serde_yaml;

/// 主题安装器
pub struct ThemeInstaller {
    theme_root: PathBuf,
}

impl ThemeInstaller {
    pub fn new(theme_root: PathBuf) -> Self {
        Self { theme_root }
    }
    
    /// 安装主题（从ZIP文件）
    pub async fn install_theme(&self, zip_content: Vec<u8>, override_existing: bool) -> Result<Theme> {
        // 1. 创建临时目录
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;
        
        // 2. 解压ZIP文件
        self.extract_zip(&zip_content, temp_dir.path())
            .context("Failed to extract ZIP file")?;
        
        // 3. 查找并加载主题manifest
        let theme_manifest_path = self.locate_theme_manifest(temp_dir.path())
            .ok_or_else(|| anyhow::anyhow!("Missing theme manifest (theme.yaml or theme.yml)"))?;
        
        let theme = self.load_theme_manifest(&theme_manifest_path)
            .context("Failed to load theme manifest")?;
        
        let theme_name = theme.metadata.name.clone();
        let theme_target_path = self.theme_root.join(&theme_name);
        
        // 4. 检查主题是否已存在
        if !override_existing && theme_target_path.exists() && !is_dir_empty(&theme_target_path)? {
            anyhow::bail!("Theme already exists: {}", theme_name);
        }
        
        // 5. 复制文件到主题目录
        let manifest_parent = theme_manifest_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid theme manifest path"))?;
        
        // 确保目标目录存在
        if theme_target_path.exists() {
            fs::remove_dir_all(&theme_target_path)
                .context("Failed to remove existing theme directory")?;
        }
        fs::create_dir_all(&theme_target_path)
            .context("Failed to create theme directory")?;
        
        copy_dir_recursive(manifest_parent, &theme_target_path)
            .context("Failed to copy theme files")?;
        
        // 6. 更新主题的location字段
        let mut theme_with_location = theme;
        theme_with_location.status = Some(flow_domain::theme::ThemeStatus {
            phase: Some(flow_domain::theme::ThemePhase::Ready),
            conditions: None,
            location: Some(theme_target_path.to_string_lossy().to_string()),
        });
        
        Ok(theme_with_location)
    }
    
    /// 升级主题
    pub async fn upgrade_theme(&self, theme_name: &str, zip_content: Vec<u8>) -> Result<Theme> {
        // 验证主题是否存在
        let theme_path = self.theme_root.join(theme_name);
        if !theme_path.exists() {
            anyhow::bail!("Theme not found: {}", theme_name);
        }
        
        // 创建临时目录
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;
        
        // 解压ZIP文件
        self.extract_zip(&zip_content, temp_dir.path())
            .context("Failed to extract ZIP file")?;
        
        // 查找并加载主题manifest
        let theme_manifest_path = self.locate_theme_manifest(temp_dir.path())
            .ok_or_else(|| anyhow::anyhow!("Missing theme manifest (theme.yaml or theme.yml)"))?;
        
        let new_theme = self.load_theme_manifest(&theme_manifest_path)
            .context("Failed to load theme manifest")?;
        
        // 验证主题名称是否匹配
        if new_theme.metadata.name != theme_name {
            anyhow::bail!(
                "Theme name mismatch: expected {}, but got {}",
                theme_name,
                new_theme.metadata.name
            );
        }
        
        // 删除旧主题文件
        fs::remove_dir_all(&theme_path)
            .context("Failed to remove old theme directory")?;
        
        // 复制新文件
        let manifest_parent = theme_manifest_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid theme manifest path"))?;
        
        fs::create_dir_all(&theme_path)
            .context("Failed to create theme directory")?;
        
        copy_dir_recursive(manifest_parent, &theme_path)
            .context("Failed to copy theme files")?;
        
        // 更新主题的location字段
        let mut theme_with_location = new_theme;
        theme_with_location.status = Some(flow_domain::theme::ThemeStatus {
            phase: Some(flow_domain::theme::ThemePhase::Ready),
            conditions: None,
            location: Some(theme_path.to_string_lossy().to_string()),
        });
        
        Ok(theme_with_location)
    }
    
    /// 解压ZIP文件
    fn extract_zip(&self, zip_content: &[u8], dest_dir: &Path) -> Result<()> {
        let mut archive = ZipArchive::new(std::io::Cursor::new(zip_content))
            .context("Failed to open ZIP archive")?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context(format!("Failed to read file {} from ZIP", i))?;
            
            let out_path = dest_dir.join(file.name());
            
            if file.name().ends_with('/') {
                // 目录
                fs::create_dir_all(&out_path)
                    .context(format!("Failed to create directory: {}", file.name()))?;
            } else {
                // 文件
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)
                        .context(format!("Failed to create parent directory for: {}", file.name()))?;
                }
                
                let mut out_file = fs::File::create(&out_path)
                    .context(format!("Failed to create file: {}", file.name()))?;
                
                std::io::copy(&mut file, &mut out_file)
                    .context(format!("Failed to extract file: {}", file.name()))?;
            }
        }
        
        Ok(())
    }
    
    /// 查找主题manifest文件（theme.yaml或theme.yml）
    pub fn locate_theme_manifest(&self, dir: &Path) -> Option<PathBuf> {
        for manifest_name in &["theme.yaml", "theme.yml"] {
            let manifest_path = dir.join(manifest_name);
            if manifest_path.exists() {
                return Some(manifest_path);
            }
        }
        None
    }
    
    /// 加载主题manifest文件
    pub fn load_theme_manifest(&self, manifest_path: &Path) -> Result<Theme> {
        let content = fs::read_to_string(manifest_path)
            .context(format!("Failed to read theme manifest: {:?}", manifest_path))?;
        
        let theme: Theme = serde_yaml::from_str(&content)
            .context(format!("Failed to parse theme manifest: {:?}", manifest_path))?;
        
        Ok(theme)
    }
}

/// 检查目录是否为空
fn is_dir_empty(dir: &Path) -> Result<bool> {
    if !dir.exists() {
        return Ok(true);
    }
    
    let mut entries = fs::read_dir(dir)
        .context("Failed to read directory")?;
    
    Ok(entries.next().is_none())
}

/// 递归复制目录
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        return Ok(());
    }
    
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(file_name);
        
        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    
    Ok(())
}

