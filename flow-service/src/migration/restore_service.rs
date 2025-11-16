use async_trait::async_trait;
use flow_api::extension::ListOptions;
use flow_infra::database::{extension_store::Model as ExtensionStoreModel, ExtensionRepository};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use zip::ZipArchive;
use std::io::Read;
use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};

/// 默认恢复服务实现
pub struct DefaultRestoreService {
    repository: Arc<dyn ExtensionRepository>,
    work_dir: PathBuf,
}

impl DefaultRestoreService {
    pub fn new(
        repository: Arc<dyn ExtensionRepository>,
        work_dir: PathBuf,
    ) -> Self {
        Self {
            repository,
            work_dir,
        }
    }
    
    /// 解压备份文件
    async fn unpack_backup<S>(&self, content: S, target: &Path) -> Result<()>
    where
        S: Stream<Item = Result<Bytes, std::io::Error>> + Send + 'static,
    {
        // 收集所有数据到内存
        let mut data = Vec::new();
        let mut stream = Box::pin(content);
        
        while let Some(chunk) = stream.try_next().await? {
            data.extend_from_slice(&chunk);
        }
        
        // 创建临时ZIP文件
        let temp_zip = target.join("backup.zip");
        fs::write(&temp_zip, &data).await?;
        
        // 解压ZIP文件
        let file = std::fs::File::open(&temp_zip)?;
        let mut archive = ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = target.join(file.name());
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent).await?;
                }
                let mut outfile = fs::File::create(&outpath).await?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                use tokio::io::AsyncWriteExt;
                outfile.write_all(&buffer).await?;
            }
        }
        
        // 删除临时ZIP文件
        fs::remove_file(&temp_zip).await?;
        
        Ok(())
    }
    
    /// 恢复扩展数据
    async fn restore_extensions(&self, backup_root: &Path) -> Result<()> {
        let extensions_path = backup_root.join("extensions.data");
        if !extensions_path.exists() {
            return Err(anyhow::anyhow!("Extensions data file not found"));
        }

        // 读取扩展数据文件
        let content = fs::read_to_string(&extensions_path).await?;

        // 先删除所有现有扩展数据
        let options = ListOptions::default();
        let stores = self.repository.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list extensions: {}", e))?;

        for store in stores {
            self.repository.delete(&store.name).await
                .map_err(|e| anyhow::anyhow!("Failed to delete extension: {}", e))?;
        }

        // 逐行读取并恢复扩展数据
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let mut store: ExtensionStoreModel = serde_json::from_str(line)?;

            // 重置版本号
            store.version = None;

            // 保存扩展数据
            self.repository.save(store).await
                .map_err(|e| anyhow::anyhow!("Failed to save extension: {}", e))?;
        }

        Ok(())
    }
    
    /// 恢复工作目录
    async fn restore_workdir(&self, backup_root: &Path) -> Result<()> {
        let workdir_backup = backup_root.join("workdir");
        if !workdir_backup.exists() {
            return Ok(());
        }
        
        // 复制工作目录内容
        copy_dir_all(&workdir_backup, &self.work_dir).await?;
        
        Ok(())
    }
}

#[async_trait]
impl super::backup_service::RestoreService for DefaultRestoreService {
    async fn restore<S>(&self, content: S) -> Result<()>
    where
        S: Stream<Item = Result<Bytes, std::io::Error>> + Send + 'static,
    {
        // 创建临时目录
        let temp_dir = tempfile::TempDir::new()?;
        
        // 解压备份文件
        self.unpack_backup(content, temp_dir.path()).await?;
        
        // 恢复扩展数据
        self.restore_extensions(temp_dir.path()).await?;
        
        // 恢复工作目录
        self.restore_workdir(temp_dir.path()).await?;
        
        Ok(())
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

