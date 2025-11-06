use flow_api::security::AuthorizationManager;
use flow_service::security::{AuthService, RoleService, UserService, PasswordService};
use flow_infra::{
    security::{JwtService, SessionService, RateLimiter},
    extension::ReactiveExtensionClient,
};
use std::sync::Arc;

/// 应用状态
/// 包含所有需要的服务实例
#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub authorization_manager: Arc<dyn AuthorizationManager>,
    pub jwt_service: Arc<JwtService>,
    pub session_service: Arc<dyn SessionService>,
    pub rate_limiter: Arc<dyn RateLimiter>,
    pub extension_client: Arc<ReactiveExtensionClient>,
    pub user_service: Arc<dyn UserService>,
    pub role_service: Arc<dyn RoleService>,
    pub password_service: Arc<dyn PasswordService>,
}

