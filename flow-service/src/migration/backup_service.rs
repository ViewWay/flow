use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions};
use flow_domain::migration::{Backup, BackupFile, BackupPhase};
use flow_infra::extension::ReactiveExtensionClient;
use flow_infra::database::ExtensionRepository;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::fs;
use zip::ZipWriter;
use zip::write::FileOptions;
use std::io::{Write, Seek};
use futures_util::Stream;

/// 备份服务trait
#[async_trait]
pub trait BackupService: Send + Sync {
    /// 创建备份
    /// 
    /// # 参数
    /// - `backup`: 备份资源对象
    async fn backup(&self, backup: Backup) -> Result<()>;
    
    /// 下载备份文件
    /// 
    /// # 参数
    /// - `backup`: 备份资源对象
    /// 
    /// # 返回
    /// - 备份文件路径
    async fn download(&self, backup: &Backup) -> Result<PathBuf>;
    
    /// 清理备份文件
    /// 
    /// # 参数
    /// - `backup`: 备份资源对象
    async fn cleanup(&self, backup: &Backup) -> Result<()>;
    
    /// 获取所有备份文件列表
    async fn get_backup_files(&self) -> Result<Vec<BackupFile>>;
    
    /// 根据文件名获取备份文件
    /// 
    /// # 参数
    /// - `filename`: 备份文件名
    async fn get_backup_file(&self, filename: &str) -> Result<Option<BackupFile>>;
}

/// 恢复服务trait
#[async_trait]
pub trait RestoreService: Send + Sync {
    /// 恢复备份
    /// 
    /// # 参数
    /// - `content`: 备份文件内容流
    async fn restore<S>(&self, content: S) -> Result<()>
    where
        S: Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send + 'static;
}

/// 默认备份服务实现
pub struct DefaultBackupService {
    extension_client: Arc<ReactiveExtensionClient>,
    repository: Arc<dyn ExtensionRepository>,
    backup_root: PathBuf,
    work_dir: PathBuf,
}

impl DefaultBackupService {
    pub fn new(
        extension_client: Arc<ReactiveExtensionClient>,
        repository: Arc<dyn ExtensionRepository>,
        backup_root: PathBuf,
        work_dir: PathBuf,
    ) -> Self {
        Self {
            extension_client,
            repository,
            backup_root,
            work_dir,
        }
    }
    
    /// 备份扩展数据
    async fn backup_extensions(&self, temp_dir: &Path) -> Result<()> {
        // 获取所有扩展数据
        let options = ListOptions::default();
        let stores = self.repository.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list extensions: {}", e))?;

        // 将扩展数据序列化为JSON并写入文件
        let extensions_path = temp_dir.join("extensions.data");
        let mut file = fs::File::create(&extensions_path).await?;

        use tokio::io::AsyncWriteExt;
        for store in stores {
            let json = serde_json::to_string(&store)?;
            file.write_all(json.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }

        Ok(())
    }
    
    /// 备份工作目录
    async fn backup_work_dir(&self, temp_dir: &Path) -> Result<()> {
        let workdir_backup = temp_dir.join("workdir");
        fs::create_dir_all(&workdir_backup).await?;
        
        // 需要备份的目录和文件
        let items_to_backup = vec![
            "themes",
            "attachments",
            "keys",
        ];
        
        for item in items_to_backup {
            let source = self.work_dir.join(item);
            if source.exists() {
                let dest = workdir_backup.join(item);
                copy_dir_all(&source, &dest).await?;
            }
        }
        
        Ok(())
    }
    
    /// 打包备份文件
    async fn package_backup(&self, temp_dir: &Path, backup: &mut Backup) -> Result<()> {
        // 生成备份文件名
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let filename = format!("halo-full-backup-{}.zip", timestamp);
        let backup_path = self.backup_root.join(&filename);
        
        // 确保备份目录存在
        fs::create_dir_all(&self.backup_root).await?;
        
        // 创建ZIP文件（使用同步API，因为zip crate不支持异步）
        use std::fs::File;
        
        let file = File::create(&backup_path)?;
        let mut zip = ZipWriter::new(std::io::BufWriter::new(file));
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        
        // 添加文件到ZIP（同步操作）
        add_dir_to_zip_sync(&mut zip, temp_dir, "", &options)?;
        
        zip.finish()?;
        
        // 获取文件大小
        let metadata = fs::metadata(&backup_path).await?;
        let size = metadata.len();
        
        // 更新备份状态
        backup.status.filename = Some(filename);
        backup.status.size = Some(size);
        backup.status.phase = BackupPhase::Succeeded;
        backup.status.completion_timestamp = Some(Utc::now());
        
        // 更新备份资源
        self.extension_client.update(backup.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to update backup: {}", e))?;
        
        Ok(())
    }
}

#[async_trait]
impl BackupService for DefaultBackupService {
    async fn backup(&self, mut backup: Backup) -> Result<()> {
        // 更新状态为运行中
        backup.status.phase = BackupPhase::Running;
        backup.status.start_timestamp = Some(Utc::now());
        self.extension_client.update(backup.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to update backup: {}", e))?;
        
        // 创建临时目录
        let temp_dir = tempfile::TempDir::new()?;
        
        // 备份扩展数据
        self.backup_extensions(temp_dir.path()).await?;
        
        // 备份工作目录
        self.backup_work_dir(temp_dir.path()).await?;
        
        // 打包备份文件
        self.package_backup(temp_dir.path(), &mut backup).await?;
        
        Ok(())
    }
    
    async fn download(&self, backup: &Backup) -> Result<PathBuf> {
        if backup.status.phase != BackupPhase::Succeeded {
            return Err(anyhow::anyhow!("Backup is not downloadable"));
        }
        
        let filename = backup.status.filename.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Backup filename not found"))?;
        
        let backup_path = self.backup_root.join(filename);
        
        if !backup_path.exists() {
            return Err(anyhow::anyhow!("Backup file not found"));
        }
        
        Ok(backup_path)
    }
    
    async fn cleanup(&self, backup: &Backup) -> Result<()> {
        if let Some(filename) = &backup.status.filename {
            let backup_path = self.backup_root.join(filename);
            
            // 安全检查：确保路径在备份目录内
            if backup_path.starts_with(&self.backup_root) {
                if backup_path.exists() {
                    fs::remove_file(&backup_path).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn get_backup_files(&self) -> Result<Vec<BackupFile>> {
        let mut backup_files = Vec::new();
        
        let mut entries = fs::read_dir(&self.backup_root).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("zip") {
                let metadata = entry.metadata().await?;
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
                    .to_string();
                
                let last_modified = metadata.modified()?;
                let duration = last_modified.duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| anyhow::anyhow!("Failed to get duration: {}", e))?;
                let last_modified = DateTime::from_timestamp(
                    duration.as_secs() as i64,
                    0
                ).unwrap_or_else(Utc::now);
                
                backup_files.push(BackupFile {
                    filename,
                    size: metadata.len(),
                    last_modified,
                });
            }
        }
        
        // 按最后修改时间降序排序
        backup_files.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        
        Ok(backup_files)
    }
    
    async fn get_backup_file(&self, filename: &str) -> Result<Option<BackupFile>> {
        let backup_path = self.backup_root.join(filename);
        
        // 安全检查
        if !backup_path.starts_with(&self.backup_root) {
            return Ok(None);
        }
        
        if !backup_path.exists() {
            return Ok(None);
        }
        
        let metadata = fs::metadata(&backup_path).await?;
        let last_modified = metadata.modified()?;
        let last_modified = DateTime::from_timestamp(
            last_modified.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64,
            0
        ).unwrap_or_else(Utc::now);
        
        Ok(Some(BackupFile {
            filename: filename.to_string(),
            size: metadata.len(),
            last_modified,
        }))
    }
}

/// 递归复制目录
fn copy_dir_all<'a>(src: &'a Path, dst: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        fs::create_dir_all(dst).await?;

        let mut entries = fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dst_path = dst.join(file_name);

            if path.is_dir() {
                copy_dir_all(&path, &dst_path).await?;
            } else {
                fs::copy(&path, &dst_path).await?;
            }
        }

        Ok(())
    })
}

/// 递归添加目录到ZIP（同步版本）
fn add_dir_to_zip_sync<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    dir: &Path,
    prefix: &str,
    options: &FileOptions<()>,
) -> Result<()> {
    use std::fs;
    use std::io::Read;
    
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let zip_path = if prefix.is_empty() {
            file_name.to_string()
        } else {
            format!("{}/{}", prefix, file_name)
        };
        
        if path.is_dir() {
            zip.add_directory(&zip_path, *options)?;
            add_dir_to_zip_sync(zip, &path, &zip_path, options)?;
        } else {
            zip.start_file(&zip_path, *options)?;
            let mut file = fs::File::open(&path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }
    
    Ok(())
}


