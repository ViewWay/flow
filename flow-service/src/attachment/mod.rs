pub mod thumbnail;
pub mod policy_service;
pub mod group_service;
pub mod policy_template_service;
pub mod shared_url;

pub use policy_service::{PolicyService, DefaultPolicyService};
pub use group_service::{GroupService, DefaultGroupService};
pub use policy_template_service::{PolicyTemplateService, DefaultPolicyTemplateService};
pub use shared_url::{SharedUrlService, DefaultSharedUrlService, SharedUrl};

use flow_domain::attachment::{Attachment, AttachmentSpec, AttachmentStatus, ThumbnailSize};
use flow_api::extension::{ExtensionClient, ListOptions, Metadata};
use flow_infra::extension::ReactiveExtensionClient;
use flow_infra::attachment::{AttachmentStorage, LocalAttachmentStorage};
use crate::attachment::thumbnail::ThumbnailService;
use async_trait::async_trait;
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;

/// Attachment服务trait
#[async_trait]
pub trait AttachmentService: Send + Sync {
    /// 上传附件
    async fn upload(
        &self,
        file_content: Vec<u8>,
        filename: String,
        media_type: Option<String>,
        owner_name: Option<String>,
        policy_name: Option<String>,
        group_name: Option<String>,
    ) -> Result<Attachment>;
    
    /// 删除附件
    async fn delete(&self, name: &str) -> Result<()>;
    
    /// 获取附件
    async fn get(&self, name: &str) -> Result<Option<Attachment>>;
    
    /// 列出附件
    async fn list(&self, options: ListOptions) -> Result<Vec<Attachment>>;
    
    /// 更新附件
    async fn update(&self, attachment: Attachment) -> Result<Attachment>;
}

/// 默认Attachment服务实现
pub struct DefaultAttachmentService {
    extension_client: Arc<ReactiveExtensionClient>,
    storage: Arc<dyn AttachmentStorage>,
    thumbnail_service: Arc<dyn ThumbnailService>,
    upload_path: PathBuf,
    base_url: String,
}

impl DefaultAttachmentService {
    pub fn new(
        extension_client: Arc<ReactiveExtensionClient>,
        storage: Arc<dyn AttachmentStorage>,
        thumbnail_service: Arc<dyn ThumbnailService>,
        upload_path: PathBuf,
        base_url: String,
    ) -> Self {
        Self {
            extension_client,
            storage,
            thumbnail_service,
            upload_path,
            base_url,
        }
    }
}

#[async_trait]
impl AttachmentService for DefaultAttachmentService {
    async fn upload(
        &self,
        file_content: Vec<u8>,
        filename: String,
        media_type: Option<String>,
        owner_name: Option<String>,
        policy_name: Option<String>,
        group_name: Option<String>,
    ) -> Result<Attachment> {
        // 1. 生成唯一文件名和路径
        let file_id = Uuid::new_v4();
        let file_path = PathBuf::from(&filename);
        let file_ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let stored_filename = format!("{}.{}", file_id, file_ext);
        let stored_path = self.upload_path.join(&stored_filename);
        
        // 2. 保存文件到存储位置
        self.storage.save(&file_content, &stored_path)?;
        
        // 3. 生成permalink
        let permalink = format!("{}/upload/{}", self.base_url.trim_end_matches('/'), stored_filename);
        
        // 4. 生成缩略图（如果是图片）
        let mut thumbnails = HashMap::new();
        if let Some(ref mime_type) = media_type {
            if self.thumbnail_service.is_image(mime_type) {
                // 生成所有尺寸的缩略图
                for size in [ThumbnailSize::Xl, ThumbnailSize::L, ThumbnailSize::M, ThumbnailSize::S] {
                    if let Ok(thumbnail_path) = self.thumbnail_service.generate_thumbnail(&stored_path, size) {
                        // 生成缩略图URL
                        let thumbnail_url = format!("{}/upload/thumbnails/{}", 
                            self.base_url.trim_end_matches('/'),
                            thumbnail_path.file_name().unwrap().to_string_lossy());
                        thumbnails.insert(size.as_str().to_string(), thumbnail_url);
                    }
                }
            }
        }
        
        // 5. 创建Attachment Extension
        let metadata = Metadata::new(file_id.to_string());
        
        let spec = AttachmentSpec {
            display_name: Some(filename.clone()),
            group_name,
            policy_name,
            owner_name,
            media_type,
            size: Some(file_content.len() as u64),
            tags: None,
        };
        
        let status = AttachmentStatus {
            permalink: Some(permalink),
            thumbnails: if thumbnails.is_empty() { None } else { Some(thumbnails) },
        };
        
        let attachment = Attachment {
            metadata,
            spec,
            status: Some(status),
        };
        
        // 6. 保存Attachment Extension
        self.extension_client.create(attachment.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to create attachment extension: {}", e))?;
        
        Ok(attachment)
    }
    
    async fn delete(&self, name: &str) -> Result<()> {
        // 1. 获取Attachment以获取文件路径
        let attachment = self.extension_client.fetch::<Attachment>(name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch attachment: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("Attachment not found: {}", name))?;
        
        // 2. 从permalink中提取文件路径
        if let Some(ref status) = attachment.status {
            if let Some(ref permalink) = status.permalink {
                // 解析permalink获取文件路径
                if let Some(relative_path) = permalink.strip_prefix(&format!("{}/upload/", self.base_url.trim_end_matches('/'))) {
                    let file_path = self.upload_path.join(relative_path);
                    
                    // 删除文件
                    if self.storage.exists(&file_path) {
                        self.storage.delete(&file_path)
                            .map_err(|e| anyhow::anyhow!("Failed to delete file: {}", e))?;
                    }
                    
                    // 删除缩略图
                    if let Some(ref thumbnails) = status.thumbnails {
                        for thumbnail_url in thumbnails.values() {
                            if let Some(thumb_relative_path) = thumbnail_url.strip_prefix(&format!("{}/upload/thumbnails/", self.base_url.trim_end_matches('/'))) {
                                let thumb_path = self.upload_path.join("thumbnails").join(thumb_relative_path);
                                if self.storage.exists(&thumb_path) {
                                    self.storage.delete(&thumb_path)
                                        .map_err(|e| anyhow::anyhow!("Failed to delete thumbnail: {}", e))?;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 3. 删除Attachment Extension
        self.extension_client.delete::<Attachment>(name).await
            .map_err(|e| anyhow::anyhow!("Failed to delete attachment extension: {}", e))?;
        
        Ok(())
    }
    
    async fn get(&self, name: &str) -> Result<Option<Attachment>> {
        self.extension_client.fetch(name).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch attachment: {}", e))
    }
    
    async fn list(&self, options: ListOptions) -> Result<Vec<Attachment>> {
        let result = self.extension_client.list(options).await
            .map_err(|e| anyhow::anyhow!("Failed to list attachments: {}", e))?;
        Ok(result.items)
    }
    
    async fn update(&self, attachment: Attachment) -> Result<Attachment> {
        self.extension_client.update(attachment).await
            .map_err(|e| anyhow::anyhow!("Failed to update attachment: {}", e))
    }
}

