use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::security::User;
use std::sync::Arc;

/// 用户服务trait
#[async_trait]
pub trait UserService: Send + Sync {
    /// 创建用户
    async fn create(&self, user: User) -> Result<User, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 更新用户
    async fn update(&self, user: User) -> Result<User, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 删除用户
    async fn delete(&self, username: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取用户
    async fn get(&self, username: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 列出用户
    async fn list(&self, options: ListOptions) -> Result<ListResult<User>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 根据邮箱查找用户
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认用户服务实现
/// 注意：由于ExtensionClient trait的限制，这里使用泛型而不是trait object
pub struct DefaultUserService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultUserService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> UserService for DefaultUserService<C> {
    async fn create(&self, user: User) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        self.client.create(user).await
    }

    async fn update(&self, user: User) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(user).await
    }

    async fn delete(&self, username: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete::<User>(username).await
    }

    async fn get(&self, username: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.fetch(username).await
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<User>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list(options).await
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        // 使用field_selector查询邮箱
        let options = ListOptions {
            field_selector: Some(format!("spec.email={}", email)),
            ..Default::default()
        };
        let result = self.list(options).await?;
        Ok(result.items.into_iter().next())
    }
}

