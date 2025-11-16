use async_trait::async_trait;
use flow_api::security::{AuthenticationProvider, AuthenticationResult, AuthRequest};
use flow_service::security::{UserConnectionService, UserService, RoleService, OAuth2UserInfo};
use flow_infra::security::SessionService;
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
}

impl OAuth2Provider {
    pub fn new(
        user_connection_service: Arc<dyn UserConnectionService>,
        user_service: Arc<dyn UserService>,
        role_service: Arc<dyn RoleService>,
        session_service: Arc<dyn SessionService>,
    ) -> Self {
        Self {
            user_connection_service,
            user_service,
            role_service,
            session_service,
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
        
        // 从Session中获取OAuth2 token
        // TODO: 实现从Session中获取OAuth2认证信息的逻辑
        // 这需要：
        // 1. 从Cookie中获取Session ID
        // 2. 从Session中获取OAuth2 token
        // 3. 验证token并获取OAuth2用户信息
        
        // 当前实现：检查是否有OAuth2相关的Session信息
        if let Some(session_id) = request.get_cookie("SESSION") {
            // TODO: 从Session中获取OAuth2认证信息
            // 这里需要实现Session中存储OAuth2 token的逻辑
            // 示例：
            // if let Some(oauth2_token) = self.session_service.get(&session_id, "oauth2_token").await? {
            //     // 验证token并获取用户信息
            //     // 查找UserConnection
            //     // 返回认证的用户
            // }
        }
        
        // 如果没有找到OAuth2认证信息，返回Unauthenticated
        // 这样其他认证提供者可以继续尝试
        Ok(AuthenticationResult::Unauthenticated)
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

