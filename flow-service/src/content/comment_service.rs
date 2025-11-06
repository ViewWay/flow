use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::content::Comment;
use std::sync::Arc;

/// Comment服务trait
#[async_trait]
pub trait CommentService: Send + Sync {
    async fn create(&self, comment: Comment) -> Result<Comment, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&self, comment: Comment) -> Result<Comment, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get(&self, name: &str) -> Result<Option<Comment>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list(&self, options: ListOptions) -> Result<ListResult<Comment>, Box<dyn std::error::Error + Send + Sync>>;
    async fn approve(&self, comment: Comment) -> Result<Comment, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_by_subject(&self, subject_ref: &flow_domain::content::SubjectRef) -> Result<Vec<Comment>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct DefaultCommentService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultCommentService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> CommentService for DefaultCommentService<C> {
    async fn create(&self, comment: Comment) -> Result<Comment, Box<dyn std::error::Error + Send + Sync>> {
        self.client.create(comment).await
    }

    async fn update(&self, comment: Comment) -> Result<Comment, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(comment).await
    }

    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete::<Comment>(name).await
    }

    async fn get(&self, name: &str) -> Result<Option<Comment>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.fetch(name).await
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<Comment>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list(options).await
    }

    async fn approve(&self, mut comment: Comment) -> Result<Comment, Box<dyn std::error::Error + Send + Sync>> {
        comment.spec.approved = Some(true);
        comment.spec.approved_time = Some(chrono::Utc::now());
        self.client.update(comment).await
    }

    async fn list_by_subject(&self, subject_ref: &flow_domain::content::SubjectRef) -> Result<Vec<Comment>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现根据主题查询评论
        let options = ListOptions::default();
        let result = self.list(options).await?;
        let comments: Vec<Comment> = result.items
            .into_iter()
            .filter(|c| {
                c.spec.subject_ref.group == subject_ref.group
                    && c.spec.subject_ref.kind == subject_ref.kind
                    && c.spec.subject_ref.name == subject_ref.name
            })
            .collect();
        Ok(comments)
    }
}

