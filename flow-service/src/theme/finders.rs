use flow_api::theme::Finder;
use crate::content::{PostService, CategoryService, TagService};
use crate::theme::ThemeService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use anyhow::Result;

/// PostFinder - 在模板中查询Post数据
/// 注意：Finder的数据查询在模板渲染前预加载，然后通过TemplateContext传递给模板
pub struct PostFinder {
    post_service: Arc<dyn PostService>,
}

impl PostFinder {
    pub fn new(post_service: Arc<dyn PostService>) -> Self {
        Self { post_service }
    }
    
    /// 根据名称获取Post（用于模板渲染前预加载）
    pub async fn get_by_name(&self, name: &str) -> Result<Value> {
        match self.post_service.get_by_username(name, "").await {
            Ok(Some(post)) => Ok(serde_json::to_value(post)?),
            Ok(None) => Ok(Value::Null),
            Err(e) => Err(anyhow::anyhow!("Failed to get post: {}", e)),
        }
    }
    
    /// 根据slug获取Post（用于模板渲染前预加载）
    pub async fn get_by_slug(&self, slug: &str) -> Result<Value> {
        use crate::content::PostQuery;
        let query = PostQuery {
            published: Some(true),
            keyword: Some(slug.to_string()),
            ..Default::default()
        };
        
        match self.post_service.list_post(query).await {
            Ok(result) => {
                // 查找匹配slug的Post
                for listed_post in result.items {
                    if listed_post.post.spec.slug == slug {
                        return Ok(serde_json::to_value(listed_post.post)?);
                    }
                }
                Ok(Value::Null)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get post by slug: {}", e)),
        }
    }
    
    /// 列出Posts（用于模板渲染前预加载）
    pub async fn list(&self, query: Option<crate::content::PostQuery>) -> Result<Value> {
        
        let query = query.unwrap_or_default();
        
        match self.post_service.list_post(query).await {
            Ok(result) => Ok(serde_json::to_value(result.items)?),
            Err(e) => Err(anyhow::anyhow!("Failed to list posts: {}", e)),
        }
    }
}

#[async_trait]
impl Finder for PostFinder {
    fn name(&self) -> &str {
        "postFinder"
    }
}

/// CategoryFinder - 在模板中查询Category数据
pub struct CategoryFinder {
    category_service: Arc<dyn CategoryService>,
}

impl CategoryFinder {
    pub fn new(category_service: Arc<dyn CategoryService>) -> Self {
        Self { category_service }
    }
    
    /// 根据名称获取Category（用于模板渲染前预加载）
    pub async fn get_by_name(&self, name: &str) -> Result<Value> {
        match self.category_service.get(name).await {
            Ok(Some(category)) => Ok(serde_json::to_value(category)?),
            Ok(None) => Ok(Value::Null),
            Err(e) => Err(anyhow::anyhow!("Failed to get category: {}", e)),
        }
    }
    
    /// 列出Categories（用于模板渲染前预加载）
    pub async fn list(&self) -> Result<Value> {
        use flow_api::extension::ListOptions;
        match self.category_service.list(ListOptions::default()).await {
            Ok(result) => Ok(serde_json::to_value(result.items)?),
            Err(e) => Err(anyhow::anyhow!("Failed to list categories: {}", e)),
        }
    }
}

#[async_trait]
impl Finder for CategoryFinder {
    fn name(&self) -> &str {
        "categoryFinder"
    }
}

/// TagFinder - 在模板中查询Tag数据
pub struct TagFinder {
    tag_service: Arc<dyn TagService>,
}

impl TagFinder {
    pub fn new(tag_service: Arc<dyn TagService>) -> Self {
        Self { tag_service }
    }
    
    /// 根据名称获取Tag（用于模板渲染前预加载）
    pub async fn get_by_name(&self, name: &str) -> Result<Value> {
        match self.tag_service.get(name).await {
            Ok(Some(tag)) => Ok(serde_json::to_value(tag)?),
            Ok(None) => Ok(Value::Null),
            Err(e) => Err(anyhow::anyhow!("Failed to get tag: {}", e)),
        }
    }
    
    /// 列出Tags（用于模板渲染前预加载）
    pub async fn list(&self) -> Result<Value> {
        use flow_api::extension::ListOptions;
        match self.tag_service.list(ListOptions::default()).await {
            Ok(result) => Ok(serde_json::to_value(result.items)?),
            Err(e) => Err(anyhow::anyhow!("Failed to list tags: {}", e)),
        }
    }
}

#[async_trait]
impl Finder for TagFinder {
    fn name(&self) -> &str {
        "tagFinder"
    }
}

/// ThemeFinder - 在模板中查询Theme数据
pub struct ThemeFinder {
    theme_service: Arc<dyn ThemeService>,
}

impl ThemeFinder {
    pub fn new(theme_service: Arc<dyn ThemeService>) -> Self {
        Self { theme_service }
    }
    
    /// 获取激活的主题（用于模板渲染前预加载）
    pub async fn get_active(&self) -> Result<Value> {
        match self.theme_service.get_active_theme().await {
            Ok(Some(theme_name)) => {
                match self.theme_service.get_theme(&theme_name).await {
                    Ok(Some(theme)) => Ok(serde_json::to_value(theme)?),
                    Ok(None) => Ok(Value::Null),
                    Err(e) => Err(anyhow::anyhow!("Failed to get theme: {}", e)),
                }
            }
            Ok(None) => Ok(Value::Null),
            Err(e) => Err(anyhow::anyhow!("Failed to get active theme: {}", e)),
        }
    }
}

#[async_trait]
impl Finder for ThemeFinder {
    fn name(&self) -> &str {
        "themeFinder"
    }
}

