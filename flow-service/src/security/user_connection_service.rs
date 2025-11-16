use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions};
use flow_api::extension::query::Condition;
use flow_domain::security::{UserConnection, UserConnectionSpec};
use std::sync::Arc;
use chrono::Utc;
use std::collections::HashMap;

/// OAuth2用户信息（从OAuth2提供者返回）
#[derive(Debug, Clone)]
pub struct OAuth2UserInfo {
    /// 提供者返回的用户唯一标识符（通常是sub或id字段）
    pub provider_user_id: String,
    /// 用户的其他属性（如email、name等）
    pub attributes: HashMap<String, serde_json::Value>,
}

impl OAuth2UserInfo {
    pub fn new(provider_user_id: String, attributes: HashMap<String, serde_json::Value>) -> Self {
        Self {
            provider_user_id,
            attributes,
        }
    }

    /// 获取用户ID（用于查找UserConnection）
    pub fn get_id(&self) -> &str {
        &self.provider_user_id
    }
}

/// 用户连接服务trait
#[async_trait]
pub trait UserConnectionService: Send + Sync {
    /// 创建用户连接
    /// 
    /// # 参数
    /// - `username`: 用户名
    /// - `registration_id`: OAuth2提供者的注册ID（如：github、google）
    /// - `oauth2_user`: OAuth2用户信息
    async fn create_user_connection(
        &self,
        username: &str,
        registration_id: &str,
        oauth2_user: &OAuth2UserInfo,
    ) -> Result<UserConnection, Box<dyn std::error::Error + Send + Sync>>;

    /// 如果存在则更新用户连接
    /// 
    /// # 参数
    /// - `registration_id`: OAuth2提供者的注册ID
    /// - `oauth2_user`: OAuth2用户信息
    /// 
    /// # 返回
    /// 如果找到并更新了连接，返回Some(UserConnection)，否则返回None
    async fn update_user_connection_if_present(
        &self,
        registration_id: &str,
        oauth2_user: &OAuth2UserInfo,
    ) -> Result<Option<UserConnection>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取用户连接
    /// 
    /// # 参数
    /// - `registration_id`: OAuth2提供者的注册ID
    /// - `username`: 用户名
    async fn get_user_connection(
        &self,
        registration_id: &str,
        username: &str,
    ) -> Result<Option<UserConnection>, Box<dyn std::error::Error + Send + Sync>>;

    /// 删除用户连接
    /// 
    /// # 参数
    /// - `registration_id`: OAuth2提供者的注册ID
    /// - `username`: 用户名
    async fn remove_user_connection(
        &self,
        registration_id: &str,
        username: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认用户连接服务实现
pub struct DefaultUserConnectionService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultUserConnectionService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> UserConnectionService for DefaultUserConnectionService<C> {
    async fn create_user_connection(
        &self,
        username: &str,
        registration_id: &str,
        oauth2_user: &OAuth2UserInfo,
    ) -> Result<UserConnection, Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否已存在连接
        if let Some(existing) = self.get_user_connection(registration_id, username).await? {
            return Err(format!(
                "User connection already exists: {}",
                existing.metadata.name
            )
            .into());
        }

        // 创建新的用户连接
        let name = format!("{}-{}", username, registration_id);
        let mut connection = UserConnection {
            metadata: flow_api::extension::Metadata::new(&name),
            spec: UserConnectionSpec {
                registration_id: registration_id.to_string(),
                username: username.to_string(),
                provider_user_id: oauth2_user.provider_user_id.clone(),
                updated_at: Some(Utc::now()),
            },
        };

        // 将OAuth2用户信息存储到annotations中
        if let Some(attrs) = connection.metadata.annotations.as_mut() {
            attrs.insert(
                "auth.halo.run/oauth2-user-info".to_string(),
                serde_json::to_string(&oauth2_user.attributes)?,
            );
        } else {
            let mut attrs = HashMap::new();
            attrs.insert(
                "auth.halo.run/oauth2-user-info".to_string(),
                serde_json::to_string(&oauth2_user.attributes)?,
            );
            connection.metadata.annotations = Some(attrs);
        }

        self.client.create(connection).await
    }

    async fn update_user_connection_if_present(
        &self,
        registration_id: &str,
        oauth2_user: &OAuth2UserInfo,
    ) -> Result<Option<UserConnection>, Box<dyn std::error::Error + Send + Sync>> {
        // 查找匹配的连接（通过registration_id和provider_user_id）
        let condition = Condition::And {
            left: Box::new(Condition::Equal {
                index_name: "spec.registrationId".to_string(),
                value: serde_json::Value::String(registration_id.to_string()),
            }),
            right: Box::new(Condition::Equal {
                index_name: "spec.providerUserId".to_string(),
                value: serde_json::Value::String(oauth2_user.provider_user_id.clone()),
            }),
        };

        let options = ListOptions {
            condition: Some(condition),
            ..Default::default()
        };

        let result = self.client.list::<UserConnection>(options).await?;
        
        if let Some(connection) = result.items.into_iter().next() {
            // 更新连接
            let mut updated = connection.clone();
            updated.spec.updated_at = Some(Utc::now());
            
            // 更新OAuth2用户信息
            if let Some(attrs) = updated.metadata.annotations.as_mut() {
                attrs.insert(
                    "auth.halo.run/oauth2-user-info".to_string(),
                    serde_json::to_string(&oauth2_user.attributes)?,
                );
            } else {
                let mut attrs = HashMap::new();
                attrs.insert(
                    "auth.halo.run/oauth2-user-info".to_string(),
                    serde_json::to_string(&oauth2_user.attributes)?,
                );
                updated.metadata.annotations = Some(attrs);
            }

            let updated_connection = self.client.update(updated).await?;
            Ok(Some(updated_connection))
        } else {
            Ok(None)
        }
    }

    async fn get_user_connection(
        &self,
        registration_id: &str,
        username: &str,
    ) -> Result<Option<UserConnection>, Box<dyn std::error::Error + Send + Sync>> {
        let condition = Condition::And {
            left: Box::new(Condition::Equal {
                index_name: "spec.registrationId".to_string(),
                value: serde_json::Value::String(registration_id.to_string()),
            }),
            right: Box::new(Condition::Equal {
                index_name: "spec.username".to_string(),
                value: serde_json::Value::String(username.to_string()),
            }),
        };

        let options = ListOptions {
            condition: Some(condition),
            ..Default::default()
        };

        let result = self.client.list::<UserConnection>(options).await?;
        Ok(result.items.into_iter().next())
    }

    async fn remove_user_connection(
        &self,
        registration_id: &str,
        username: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(connection) = self.get_user_connection(registration_id, username).await? {
            self.client.delete::<UserConnection>(&connection.metadata.name).await?;
        }
        Ok(())
    }
}

