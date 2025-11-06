use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_api::extension::query::{Condition, queries};
use flow_domain::content::{Post, PostSpec, PostPhase, Snapshot, SubjectRef};
use flow_domain::content::constant;
use std::sync::Arc;
use chrono::Utc;
use serde_json::Value;
use crate::content::patch_utils;

/// Post请求，包含Post和内容
#[derive(Debug, Clone)]
pub struct PostRequest {
    pub post: Post,
    pub content: Option<ContentRequest>,
}

/// 内容请求
#[derive(Debug, Clone)]
pub struct ContentRequest {
    pub raw: String,
    pub content: String,
    pub raw_type: String,
}

/// Post查询参数
#[derive(Debug, Clone, Default)]
pub struct PostQuery {
    pub published: Option<bool>,
    pub owner: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub keyword: Option<String>,
    pub visible: Option<flow_domain::content::VisibleEnum>,
    pub page: Option<u32>,
    pub size: Option<u32>,
}

impl PostQuery {
    /// 构建ListOptions
    pub fn to_list_options(&self) -> ListOptions {
        let mut options = ListOptions::default();
        let mut condition = Condition::empty();
        
        // 构建标签选择器
        let mut label_selectors = Vec::new();
        if let Some(published) = self.published {
            label_selectors.push(format!("{}={}", constant::POST_PUBLISHED_LABEL, published));
        }
        if let Some(ref owner) = self.owner {
            label_selectors.push(format!("{}={}", constant::POST_OWNER_LABEL, owner));
        }
        if !label_selectors.is_empty() {
            options.label_selector = Some(label_selectors.join(","));
        }
        
        // 关键词搜索（在status.excerpt、spec.slug、spec.title中搜索）
        if let Some(ref keyword) = self.keyword {
            if !keyword.is_empty() {
                let keyword_condition = Condition::Or {
                    left: Box::new(Condition::Contains {
                        index_name: "status.excerpt".to_string(),
                        value: keyword.clone(),
                    }),
                    right: Box::new(Condition::Or {
                        left: Box::new(Condition::Contains {
                            index_name: "spec.slug".to_string(),
                            value: keyword.clone(),
                        }),
                        right: Box::new(Condition::Contains {
                            index_name: "spec.title".to_string(),
                            value: keyword.clone(),
                        }),
                    }),
                };
                condition = condition.and(keyword_condition);
            }
        }
        
        // 分类过滤
        if let Some(ref category) = self.category {
            condition = condition.and(Condition::Contains {
                index_name: "spec.categories".to_string(),
                value: category.clone(),
            });
        }
        
        // 标签过滤
        if let Some(ref tag) = self.tag {
            condition = condition.and(Condition::Contains {
                index_name: "spec.tags".to_string(),
                value: tag.clone(),
            });
        }
        
        // 可见性过滤
        if let Some(visible) = self.visible {
            condition = condition.and(Condition::Equal {
                index_name: "spec.visible".to_string(),
                value: Value::String(format!("{:?}", visible)),
            });
        }
        
        // 所有者过滤
        if let Some(ref owner) = self.owner {
            condition = condition.and(Condition::Equal {
                index_name: "spec.owner".to_string(),
                value: Value::String(owner.clone()),
            });
        }
        
        // 设置查询条件
        if !matches!(condition, Condition::Empty) {
            options.condition = Some(condition);
        }
        
        // 设置分页
        options.page = self.page;
        options.size = self.size;
        
        options
    }
}

/// 列出的Post（简化版，用于列表显示）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListedPost {
    pub post: Post,
    // 可以添加其他列表显示需要的字段
}

/// 内容包装器
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentWrapper {
    #[serde(rename = "snapshotName")]
    pub snapshot_name: String,
    pub raw: String,
    pub content: String,
    #[serde(rename = "rawType")]
    pub raw_type: String,
}

/// Post服务trait
#[async_trait]
pub trait PostService: Send + Sync {
    /// 列出文章
    async fn list_post(&self, query: PostQuery) -> Result<ListResult<ListedPost>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 创建草稿
    async fn draft_post(&self, request: PostRequest) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 更新文章
    async fn update_post(&self, request: PostRequest) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 更新文章（直接传入Post对象）
    async fn update_by(&self, post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取头部内容（最新版本）
    async fn get_head_content(&self, post_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取发布内容
    async fn get_release_content(&self, post_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取指定快照的内容
    async fn get_content(&self, snapshot_name: &str, base_snapshot_name: Option<&str>) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 发布文章
    async fn publish(&self, post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 取消发布文章
    async fn unpublish(&self, post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 根据用户名获取文章
    async fn get_by_username(&self, post_name: &str, username: &str) -> Result<Option<Post>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 恢复到指定快照
    async fn revert_to_snapshot(&self, post_name: &str, snapshot_name: &str) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 删除内容（删除指定快照）
    async fn delete_content(&self, post_name: &str, snapshot_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 回收文章（移到回收站）
    async fn recycle(&self, post_name: &str, username: &str) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认Post服务实现
pub struct DefaultPostService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultPostService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> PostService for DefaultPostService<C> {
    async fn list_post(&self, query: PostQuery) -> Result<ListResult<ListedPost>, Box<dyn std::error::Error + Send + Sync>> {
        let options = query.to_list_options();
        let result = self.client.list::<Post>(options).await?;
        
        let listed_posts: Vec<ListedPost> = result.items
            .into_iter()
            .map(|post| ListedPost { post })
            .collect();
        
        Ok(ListResult::new(listed_posts, result.total, result.page, result.size))
    }

    async fn draft_post(&self, request: PostRequest) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现草稿创建逻辑
        // 1. 创建Post
        // 2. 如果提供了内容，创建Snapshot
        // 3. 更新Post的headSnapshot和baseSnapshot
        let mut post = request.post;
        
        // 设置默认值
        if post.spec.deleted.is_none() {
            post.spec.deleted = Some(false);
        }
        if post.spec.publish.is_none() {
            post.spec.publish = Some(false);
        }
        if post.spec.pinned.is_none() {
            post.spec.pinned = Some(false);
        }
        if post.spec.allow_comment.is_none() {
            post.spec.allow_comment = Some(true);
        }
        if post.spec.visible.is_none() {
            post.spec.visible = Some(flow_domain::content::VisibleEnum::Public);
        }
        if post.spec.priority.is_none() {
            post.spec.priority = Some(0);
        }
        
        // 创建Post
        let created_post = self.client.create(post).await?;
        
        // TODO: 如果提供了内容，创建Snapshot
        // if let Some(content) = request.content {
        //     // 创建Snapshot逻辑
        // }
        
        Ok(created_post)
    }

    async fn update_post(&self, request: PostRequest) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现更新逻辑
        // 1. 获取现有Post
        // 2. 更新Post字段
        // 3. 如果提供了内容，创建新的Snapshot
        // 4. 更新Post
        let mut post = request.post;
        self.client.update(post).await
    }

    async fn update_by(&self, post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(post).await
    }

    async fn get_head_content(&self, post_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        let post = self.client.fetch::<Post>(post_name).await?
            .ok_or_else(|| "Post not found")?;
        
        let head_snapshot = post.spec.head_snapshot.as_ref()
            .ok_or_else(|| "Head snapshot not found")?;
        let base_snapshot = post.spec.base_snapshot.as_ref()
            .ok_or_else(|| "Base snapshot not found")?;
        
        self.get_content(head_snapshot, Some(base_snapshot)).await
    }

    async fn get_release_content(&self, post_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        let post = self.client.fetch::<Post>(post_name).await?
            .ok_or_else(|| "Post not found")?;
        
        let release_snapshot = post.spec.release_snapshot.as_ref()
            .ok_or_else(|| "Release snapshot not found")?;
        let base_snapshot = post.spec.base_snapshot.as_ref()
            .ok_or_else(|| "Base snapshot not found")?;
        
        self.get_content(release_snapshot, Some(base_snapshot)).await
    }

    async fn get_content(&self, snapshot_name: &str, base_snapshot_name: Option<&str>) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        let base_snapshot_name = base_snapshot_name.ok_or_else(|| "Base snapshot name is required")?;
        
        // 获取base snapshot
        let base_snapshot = self.client.fetch::<Snapshot>(base_snapshot_name).await?
            .ok_or_else(|| "Base snapshot not found")?;
        
        // 检查是否是base snapshot
        if !base_snapshot.is_base_snapshot() {
            return Err("The snapshot is not a base snapshot".into());
        }
        
        // 如果snapshot_name等于base_snapshot_name，直接返回base snapshot的内容
        if snapshot_name == base_snapshot_name {
            let raw = base_snapshot.spec.raw_patch.as_deref().unwrap_or("");
            let content = base_snapshot.spec.content_patch.as_deref().unwrap_or("");
            return Ok(ContentWrapper {
                snapshot_name: base_snapshot.metadata.name.clone(),
                raw: raw.to_string(),
                content: content.to_string(),
                raw_type: base_snapshot.spec.raw_type.clone(),
            });
        }
        
        // 获取patch snapshot
        let patch_snapshot = self.client.fetch::<Snapshot>(snapshot_name).await?
            .ok_or_else(|| "Snapshot not found")?;
        
        // 应用patch
        let base_raw = base_snapshot.spec.raw_patch.as_deref().unwrap_or("");
        let base_content = base_snapshot.spec.content_patch.as_deref().unwrap_or("");
        
        let raw_patch = patch_snapshot.spec.raw_patch.as_deref().unwrap_or("");
        let content_patch = patch_snapshot.spec.content_patch.as_deref().unwrap_or("");
        
        let patched_raw = if raw_patch.is_empty() {
            base_raw.to_string()
        } else {
            patch_utils::apply_patch(base_raw, raw_patch)?
        };
        
        let patched_content = if content_patch.is_empty() {
            base_content.to_string()
        } else {
            patch_utils::apply_patch(base_content, content_patch)?
        };
        
        Ok(ContentWrapper {
            snapshot_name: patch_snapshot.metadata.name.clone(),
            raw: patched_raw,
            content: patched_content,
            raw_type: patch_snapshot.spec.raw_type.clone(),
        })
    }

    async fn publish(&self, mut post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        // 设置发布标签
        if post.metadata.labels.is_none() {
            post.metadata.labels = Some(std::collections::HashMap::new());
        }
        if let Some(ref mut labels) = post.metadata.labels {
            labels.insert(constant::POST_PUBLISHED_LABEL.to_string(), "true".to_string());
        }
        
        // 设置发布状态
        post.spec.publish = Some(true);
        if post.spec.publish_time.is_none() {
            post.spec.publish_time = Some(Utc::now());
        }
        
        // 更新状态
        if post.status.is_none() {
            post.status = Some(Default::default());
        }
        if let Some(ref mut status) = post.status {
            status.phase = Some(PostPhase::Published);
        }
        
        self.client.update(post).await
    }

    async fn unpublish(&self, mut post: Post) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        // 移除发布标签
        if let Some(ref mut labels) = post.metadata.labels {
            labels.insert(constant::POST_PUBLISHED_LABEL.to_string(), "false".to_string());
        }
        
        // 设置发布状态
        post.spec.publish = Some(false);
        
        // 更新状态
        if let Some(ref mut status) = post.status {
            status.phase = Some(PostPhase::Draft);
        }
        
        self.client.update(post).await
    }

    async fn get_by_username(&self, post_name: &str, _username: &str) -> Result<Option<Post>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现根据用户名获取文章（需要检查权限）
        self.client.fetch::<Post>(post_name).await
    }

    async fn revert_to_snapshot(&self, post_name: &str, snapshot_name: &str) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        // 获取Post
        let mut post = self.client.fetch::<Post>(post_name).await?
            .ok_or_else(|| "Post not found")?;
        
        // 检查head snapshot是否与目标snapshot相同
        if let Some(ref head_snapshot) = post.spec.head_snapshot {
            if head_snapshot == snapshot_name {
                // 已经是目标snapshot，无需恢复
                return Ok(post);
            }
        }
        
        // 获取base snapshot
        let base_snapshot_name = post.spec.base_snapshot.as_ref()
            .ok_or_else(|| "Base snapshot not found")?;
        
        // 获取目标snapshot的内容
        let content_wrapper = self.get_content(snapshot_name, Some(base_snapshot_name)).await?;
        
        // 创建新的snapshot作为head snapshot
        // TODO: 这里应该创建新的Snapshot，但为了简化，我们直接更新head_snapshot
        // 实际实现中应该创建新的Snapshot并更新Post的head_snapshot
        post.spec.head_snapshot = Some(snapshot_name.to_string());
        
        // 更新并发布
        let updated_post = self.client.update(post).await?;
        self.publish(updated_post).await
    }

    async fn delete_content(&self, post_name: &str, snapshot_name: &str) -> Result<ContentWrapper, Box<dyn std::error::Error + Send + Sync>> {
        // 获取Post
        let mut post = self.client.fetch::<Post>(post_name).await?
            .ok_or_else(|| "Post not found")?;
        
        // 如果删除的是head snapshot，将head设置为release snapshot
        if let Some(ref head_snapshot) = post.spec.head_snapshot {
            if head_snapshot == snapshot_name {
                if let Some(ref release_snapshot) = post.spec.release_snapshot {
                    post.spec.head_snapshot = Some(release_snapshot.clone());
                    post = self.client.update(post).await?;
                } else {
                    return Err("Cannot delete head snapshot: no release snapshot available".into());
                }
            }
        }
        
        // 检查是否是release snapshot或base snapshot
        if let Some(ref release_snapshot) = post.spec.release_snapshot {
            if release_snapshot == snapshot_name {
                return Err("Cannot delete release snapshot. Please revert to another snapshot first.".into());
            }
        }
        
        if let Some(ref base_snapshot) = post.spec.base_snapshot {
            if base_snapshot == snapshot_name {
                return Err("The first snapshot cannot be deleted.".into());
            }
        }
        
        // 获取要删除的snapshot内容（用于返回）
        let base_snapshot_name = post.spec.base_snapshot.as_ref()
            .ok_or_else(|| "Base snapshot not found")?;
        let content_wrapper = self.get_content(snapshot_name, Some(base_snapshot_name)).await?;
        
        // 删除snapshot
        self.client.delete::<Snapshot>(snapshot_name).await?;
        
        Ok(content_wrapper)
    }

    async fn recycle(&self, post_name: &str, _username: &str) -> Result<Post, Box<dyn std::error::Error + Send + Sync>> {
        // 获取Post
        let mut post = self.client.fetch::<Post>(post_name).await?
            .ok_or_else(|| "Post not found")?;
        
        // 设置删除标签
        if post.metadata.labels.is_none() {
            post.metadata.labels = Some(std::collections::HashMap::new());
        }
        if let Some(ref mut labels) = post.metadata.labels {
            labels.insert(constant::POST_DELETED_LABEL.to_string(), "true".to_string());
        }
        
        post.spec.deleted = Some(true);
        
        self.client.update(post).await
    }
}

