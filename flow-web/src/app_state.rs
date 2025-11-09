use flow_api::security::AuthorizationManager;
use flow_service::security::{AuthService, RoleService, UserService, PasswordService};
use flow_service::content::{PostService, SinglePageService, CommentService, CategoryService, TagService};
use flow_service::search::SearchService;
use flow_service::attachment::{AttachmentService, PolicyService, GroupService, SharedUrlService};
use flow_service::theme::ThemeService;
use flow_infra::{
    security::{JwtService, SessionService, RateLimiter},
    extension::ReactiveExtensionClient,
    theme::{ThemeResolver, TemplateEngineManager},
    websocket::WebSocketEndpointManager,
};
use std::sync::Arc;
use std::path::PathBuf;

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
    pub post_service: Arc<dyn PostService>,
    pub single_page_service: Arc<dyn SinglePageService>,
    pub comment_service: Arc<dyn CommentService>,
    pub category_service: Arc<dyn CategoryService>,
    pub tag_service: Arc<dyn TagService>,
    pub search_service: Arc<dyn SearchService>,
    pub attachment_service: Arc<dyn AttachmentService>,
    pub policy_service: Arc<dyn PolicyService>,
    pub group_service: Arc<dyn GroupService>,
    pub shared_url_service: Arc<dyn SharedUrlService>,
    pub theme_service: Arc<dyn ThemeService>,
    pub theme_root: PathBuf,
    pub theme_resolver: Arc<ThemeResolver>,
    pub template_engine_manager: Arc<TemplateEngineManager>,
    pub websocket_manager: Arc<WebSocketEndpointManager>,
}

