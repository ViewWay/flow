use async_trait::async_trait;
use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest, AuthenticatedUser};
use flow_service::security::{UserConnectionService, UserService, RoleService, OAuth2UserInfo};
use flow_infra::security::{SessionService, OAuth2TokenCache};
use std::sync::Arc;
use std::collections::HashMap;
use url::Url;
use serde_json::Value;
use reqwest;
use uuid::Uuid;

/// OAuth2认证提供者
/// 
/// 注意：OAuth2的完整流程（授权URL生成、回调处理）需要通过专门的路由来处理。
/// 这个Provider主要用于处理已经通过OAuth2流程获得的认证信息（如Session中的OAuth2 token）。
pub struct OAuth2Provider {
    user_connection_service: Arc<dyn UserConnectionService>,
    user_service: Arc<dyn UserService>,
    role_service: Arc<dyn RoleService>,
    session_service: Arc<dyn SessionService>,
    oauth2_token_cache: Arc<dyn OAuth2TokenCache>,
}

impl OAuth2Provider {
    pub fn new(
        user_connection_service: Arc<dyn UserConnectionService>,
        user_service: Arc<dyn UserService>,
        role_service: Arc<dyn RoleService>,
        session_service: Arc<dyn SessionService>,
        oauth2_token_cache: Arc<dyn OAuth2TokenCache>,
    ) -> Self {
        Self {
            user_connection_service,
            user_service,
            role_service,
            session_service,
            oauth2_token_cache,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for OAuth2Provider {
    async fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, Box<dyn std::error::Error + Send + Sync>> {
        // OAuth2认证流程通常通过Session来处理
        // 1. 检查Session中是否有OAuth2认证信息
        // 2. 如果有，验证并查找UserConnection
        // 3. 如果找到UserConnection，返回认证的用户
        
        // 从Cookie中获取Session ID
        let session_id = match request.get_cookie("SESSION") {
            Some(id) => id,
            None => {
                // 没有Session，返回Unauthenticated
                return Ok(AuthenticationResult::Unauthenticated);
            }
        };
        
        // 从OAuth2 token缓存中获取token信息
        let token_info = match self.oauth2_token_cache.get_token(&session_id).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                // 没有OAuth2 token，返回Unauthenticated
                return Ok(AuthenticationResult::Unauthenticated);
            }
            Err(e) => {
                tracing::warn!("Failed to get OAuth2 token from cache: {}", e);
                return Ok(AuthenticationResult::Unauthenticated);
            }
        };
        
        // 查找UserConnection
        let oauth2_user_info = OAuth2UserInfo::new(
            token_info.provider_user_id.clone(),
            token_info.attributes.clone(),
        );
        
        // 查找UserConnection（使用update_user_connection_if_present方法）
        let user_connection = match self.user_connection_service
            .update_user_connection_if_present(
                &token_info.registration_id,
                &oauth2_user_info,
            )
            .await
        {
            Ok(Some(conn)) => conn,
            Ok(None) => {
                // 没有找到UserConnection，返回Unauthenticated
                // 这可能是因为用户还没有绑定OAuth2账号
                return Ok(AuthenticationResult::Unauthenticated);
            }
            Err(e) => {
                tracing::error!("Failed to find UserConnection: {}", e);
                return Ok(AuthenticationResult::Unauthenticated);
            }
        };
        
        // 从UserConnection中获取用户名
        let username = user_connection.spec.username.clone();
        
        // 查找用户
        let user = match self.user_service.get(&username).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                tracing::warn!("User not found: {}", username);
                return Ok(AuthenticationResult::Unauthenticated);
            }
            Err(e) => {
                tracing::error!("Failed to find user: {}", e);
                return Ok(AuthenticationResult::Unauthenticated);
            }
        };
        
        // 获取用户角色
        let roles = match self.role_service.get_user_roles(&username).await {
            Ok(roles) => roles,
            Err(e) => {
                tracing::warn!("Failed to get user roles: {}", e);
                vec![] // 如果没有角色，返回空列表
            }
        };
        
        // 创建AuthenticatedUser
        let authenticated_user = AuthenticatedUser {
            username: user.metadata.name.clone(),
            roles,
            authorities: vec![], // authorities会在AuthenticatedUser::new中自动生成
        };
        
        // 返回认证成功
        Ok(AuthenticationResult::Authenticated(authenticated_user))
    }

    fn priority(&self) -> u32 {
        15 // OAuth2优先级中等
    }
}

impl Default for OAuth2Provider {
    fn default() -> Self {
        // Default实现不应该被使用，因为需要依赖注入
        // 这里提供一个占位实现以避免编译错误
        panic!("OAuth2Provider::default() should not be called. Use OAuth2Provider::new() instead.")
    }
}

/// OAuth2配置
/// 用于配置OAuth2客户端（如GitHub、Google等）
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// 客户端ID
    pub client_id: String,
    /// 客户端密钥
    pub client_secret: String,
    /// 授权端点URL
    pub auth_url: String,
    /// Token端点URL
    pub token_url: String,
    /// 用户信息端点URL
    pub user_info_url: String,
    /// 重定向URI
    pub redirect_uri: String,
    /// 作用域
    pub scopes: Vec<String>,
}

impl OAuth2Config {
    pub fn new(
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        user_info_url: String,
        redirect_uri: String,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            auth_url,
            token_url,
            user_info_url,
            redirect_uri,
            scopes,
        }
    }
}

/// OAuth2客户端（用于处理OAuth2流程）
/// 实现完整的OAuth2流程
pub struct OAuth2Client {
    config: OAuth2Config,
    http_client: reqwest::Client,
}

impl OAuth2Client {
    pub fn new(config: OAuth2Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            config,
            http_client: reqwest::Client::new(),
        })
    }

    /// 生成授权URL
    /// 返回授权URL和CSRF state token
    pub fn get_authorize_url(&self) -> Result<(Url, String), Box<dyn std::error::Error + Send + Sync>> {
        let state = Uuid::new_v4().to_string();
        let mut auth_url = Url::parse(&self.config.auth_url)?;
        
        auth_url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &self.config.scopes.join(" "))
            .append_pair("state", &state);
        
        Ok((auth_url, state))
    }

    /// 交换授权码获取Token
    pub async fn exchange_code(&self, code: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut params = std::collections::HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("redirect_uri", &self.config.redirect_uri);
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);
        
        let response = self.http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to exchange code: {}", response.status()).into());
        }
        
        let json: Value = response.json().await?;
        let access_token = json.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or("Missing access_token in OAuth2 response")?;
        
        Ok(access_token.to_string())
    }

    /// 获取用户信息
    pub async fn get_user_info(&self, token: &str) -> Result<OAuth2UserInfo, Box<dyn std::error::Error + Send + Sync>> {
        // 构建请求
        let response = self.http_client
            .get(&self.config.user_info_url)
            .bearer_auth(token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get user info: {}", response.status()).into());
        }
        
        let json: Value = response.json().await?;
        
        // 解析用户信息（根据不同的OAuth2提供者，字段可能不同）
        // 这里使用通用的字段名，实际使用时可能需要根据提供者调整
        let provider_user_id = json.get("id")
            .or_else(|| json.get("sub"))
            .or_else(|| json.get("user_id"))
            .and_then(|v| v.as_str())
            .ok_or("Missing user ID in OAuth2 response")?
            .to_string();
        
        // 提取其他属性
        let mut attributes = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                attributes.insert(key.clone(), value.clone());
            }
        }
        
        Ok(OAuth2UserInfo::new(provider_user_id, attributes))
    }
}

