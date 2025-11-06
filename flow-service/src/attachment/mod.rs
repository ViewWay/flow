pub mod thumbnail;

use flow_domain::attachment::Attachment;
use flow_api::extension::{ExtensionClient, ListOptions};
use flow_infra::extension::ReactiveExtensionClient;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

/// Attachment服务trait
#[async_trait]
pub trait AttachmentService: Send + Sync {
    /// 上传附件
    async fn upload(&self, file_content: Vec<u8>, filename: String, media_type: Option<String>, owner_name: Option<String>) -> Result<Attachment>;
    
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
}

impl DefaultAttachmentService {
    pub fn new(extension_client: Arc<ReactiveExtensionClient>) -> Self {
        Self { extension_client }
    }
}

#[async_trait]
impl AttachmentService for DefaultAttachmentService {
    async fn upload(&self, _file_content: Vec<u8>, _filename: String, _media_type: Option<String>, _owner_name: Option<String>) -> Result<Attachment> {
        // TODO: 实现文件上传逻辑
        // 1. 保存文件到存储位置
        // 2. 生成permalink
        // 3. 生成缩略图（如果是图片）
        // 4. 创建Attachment Extension
        anyhow::bail!("Attachment upload not implemented yet")
    }
    
    async fn delete(&self, name: &str) -> Result<()> {
        // TODO: 实现文件删除逻辑
        // 1. 删除文件
        // 2. 删除缩略图
        // 3. 删除Attachment Extension
        self.extension_client.delete::<Attachment>(name).await
            .map_err(|e| anyhow::anyhow!("Failed to delete attachment: {}", e))
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

