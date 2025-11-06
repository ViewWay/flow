use flow_api::search::HaloDocument;
use flow_domain::content::{Post, SinglePage};
use crate::content::ContentWrapper;

/// 文档类型常量
pub mod doc_type {
    pub const POST: &str = "post.content.halo.run";
    pub const SINGLE_PAGE: &str = "singlepage.content.halo.run";
}

/// 文档转换器，将内容实体转换为HaloDocument
pub struct DocumentConverter;

impl DocumentConverter {
    /// 将Post转换为HaloDocument
    pub fn convert_post(post: &Post, content: &ContentWrapper) -> HaloDocument {
        let spec = &post.spec;
        let status = post.status.as_ref();
        
        // 构建文档ID：{doc_type}-{metadata_name}
        let doc_id = format!("{}-{}", doc_type::POST, post.metadata.name);
        
        // 获取标题
        let title = spec.title.clone();
        
        // 获取描述（从status.excerpt）
        let description = status
            .and_then(|s| s.excerpt.as_ref())
            .map(|s| s.clone());
        
        // 获取内容（安全内容，无HTML标签）
        let content_text = content.content.clone();
        
        // 获取分类和标签
        let categories = spec.categories.clone();
        let tags = spec.tags.clone();
        
        // 获取发布状态
        let published = post.is_published();
        
        // 获取回收状态
        let recycled = post.is_deleted();
        
        // 获取公开状态
        let exposed = post.is_public();
        
        // 获取所有者
        let owner_name = spec.owner.clone().unwrap_or_default();
        
        // 获取永久链接
        let permalink = status
            .and_then(|s| s.permalink.as_ref())
            .map(|s| s.clone())
            .unwrap_or_default();
        
        // 获取时间戳
        let creation_timestamp = post.metadata.creation_timestamp;
        let update_timestamp = spec.publish_time.or(creation_timestamp);
        
        HaloDocument {
            id: doc_id,
            metadata_name: post.metadata.name.clone(),
            annotations: post.metadata.annotations.clone(),
            title,
            description,
            content: content_text,
            categories,
            tags,
            published,
            recycled,
            exposed,
            owner_name,
            creation_timestamp,
            update_timestamp,
            permalink,
            doc_type: doc_type::POST.to_string(),
        }
    }
    
    /// 将SinglePage转换为HaloDocument
    pub fn convert_single_page(page: &SinglePage, content: &ContentWrapper) -> HaloDocument {
        let spec = &page.spec;
        let status = page.status.as_ref();
        
        // 构建文档ID：{doc_type}-{metadata_name}
        let doc_id = format!("{}-{}", doc_type::SINGLE_PAGE, page.metadata.name);
        
        // 获取标题
        let title = spec.title.clone();
        
        // 获取描述（从status.excerpt）
        let description = status
            .and_then(|s| s.excerpt.as_ref())
            .map(|s| s.clone());
        
        // 获取内容（安全内容，无HTML标签）
        let content_text = content.content.clone();
        
        // SinglePage没有分类和标签
        let categories = None;
        let tags = None;
        
        // 获取发布状态
        let published = page.is_published();
        
        // 获取回收状态
        let recycled = spec.deleted.unwrap_or(false);
        
        // 获取公开状态
        let exposed = matches!(spec.visible, Some(flow_domain::content::VisibleEnum::Public) | None);
        
        // 获取所有者
        let owner_name = spec.owner.clone().unwrap_or_default();
        
        // 获取永久链接
        let permalink = status
            .and_then(|s| s.permalink.as_ref())
            .map(|s| s.clone())
            .unwrap_or_default();
        
        // 获取时间戳
        let creation_timestamp = page.metadata.creation_timestamp;
        let update_timestamp = spec.publish_time.or(creation_timestamp);
        
        HaloDocument {
            id: doc_id,
            metadata_name: page.metadata.name.clone(),
            annotations: page.metadata.annotations.clone(),
            title,
            description,
            content: content_text,
            categories,
            tags,
            published,
            recycled,
            exposed,
            owner_name,
            creation_timestamp,
            update_timestamp,
            permalink,
            doc_type: doc_type::SINGLE_PAGE.to_string(),
        }
    }
    
    /// 生成Post的文档ID
    pub fn post_doc_id(post_name: &str) -> String {
        format!("{}-{}", doc_type::POST, post_name)
    }
    
    /// 生成SinglePage的文档ID
    pub fn single_page_doc_id(page_name: &str) -> String {
        format!("{}-{}", doc_type::SINGLE_PAGE, page_name)
    }
}

