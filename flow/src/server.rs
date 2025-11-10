use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use flow_api::security::AuthorizationManager;
use flow_service::security::{
    AuthService, RoleService, UserService, PasswordService, DefaultPasswordService,
};
use flow_service::content::{
    PostService, DefaultPostService, SearchIndexingPostService,
    SinglePageService, DefaultSinglePageService, SearchIndexingSinglePageService,
    CommentService, DefaultCommentService,
    CategoryService, DefaultCategoryService,
    TagService, DefaultTagService,
};
use flow_service::theme::{ThemeService, DefaultThemeService};
use flow_service::notification::{NotificationService, NotificationCenter, DefaultNotificationService, DefaultNotificationCenter, NotificationSender};
use async_trait::async_trait;
use flow_infra::{
    database::DatabaseManager,
    security::{JwtService, SessionService, RateLimiter},
    extension::ReactiveExtensionClient,
    search::TantivySearchEngine,
    index::{IndicesManager, FulltextFieldMapping},
};
use flow_api::search::SearchEngine;
use flow_service::search::{SearchService, DefaultSearchService};
use flow_web::{AppState, openapi::ApiDoc};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;

/// 创建应用路由
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1alpha1/health", get(health_check))
        // 认证相关路由
        .route("/api/v1alpha1/login", post(flow_web::login))
        .route("/api/v1alpha1/logout", post(flow_web::logout))
        .route("/api/v1alpha1/users/-/current", get(flow_web::get_current_user))
        // 用户管理路由
        .route("/api/v1alpha1/users", get(flow_web::list_users).post(flow_web::create_user))
        .route("/api/v1alpha1/users/:name", get(flow_web::get_user).put(flow_web::update_user).delete(flow_web::delete_user))
        .route("/api/v1alpha1/users/:name/roles", post(flow_web::grant_user_roles))
        // 角色管理路由
        .route("/api/v1alpha1/roles", get(flow_web::list_roles).post(flow_web::create_role))
        .route("/api/v1alpha1/roles/:name", get(flow_web::get_role))
        // 角色绑定路由
        .route("/api/v1alpha1/rolebindings", get(flow_web::list_role_bindings).post(flow_web::create_role_binding))
        .route("/api/v1alpha1/rolebindings/:name", get(flow_web::get_role_binding).delete(flow_web::delete_role_binding))
        // Post管理路由
        .route("/api/v1alpha1/posts", get(flow_web::list_posts).post(flow_web::create_post))
        .route("/api/v1alpha1/posts/:name", get(flow_web::get_post).put(flow_web::update_post).delete(flow_web::delete_post))
        .route("/api/v1alpha1/posts/:name/publish", axum::routing::put(flow_web::publish_post))
        .route("/api/v1alpha1/posts/:name/unpublish", axum::routing::put(flow_web::unpublish_post))
        .route("/api/v1alpha1/posts/:name/recycle", axum::routing::put(flow_web::recycle_post))
        .route("/api/v1alpha1/posts/:name/head-content", get(flow_web::get_post_head_content))
        .route("/api/v1alpha1/posts/:name/release-content", get(flow_web::get_post_release_content))
        .route("/api/v1alpha1/posts/:name/content", get(flow_web::get_post_content).delete(flow_web::delete_post_content))
        .route("/api/v1alpha1/posts/:name/revert-content", axum::routing::put(flow_web::revert_post_to_snapshot))
        // SinglePage管理路由
        .route("/api/v1alpha1/singlepages", get(flow_web::list_single_pages).post(flow_web::create_single_page))
        .route("/api/v1alpha1/singlepages/:name", get(flow_web::get_single_page).put(flow_web::update_single_page).delete(flow_web::delete_single_page))
        .route("/api/v1alpha1/singlepages/:name/publish", axum::routing::put(flow_web::publish_single_page))
        .route("/api/v1alpha1/singlepages/:name/unpublish", axum::routing::put(flow_web::unpublish_single_page))
        // Comment管理路由
        .route("/api/v1alpha1/comments", get(flow_web::list_comments).post(flow_web::create_comment))
        .route("/api/v1alpha1/comments/:name", get(flow_web::get_comment).put(flow_web::update_comment).delete(flow_web::delete_comment))
        .route("/api/v1alpha1/comments/:name/approve", axum::routing::put(flow_web::approve_comment))
        // Category管理路由
        .route("/api/v1alpha1/categories", get(flow_web::list_categories).post(flow_web::create_category))
        .route("/api/v1alpha1/categories/:name", get(flow_web::get_category).put(flow_web::update_category).delete(flow_web::delete_category))
        // Tag管理路由
        .route("/api/v1alpha1/tags", get(flow_web::list_tags).post(flow_web::create_tag))
        .route("/api/v1alpha1/tags/:name", get(flow_web::get_tag).put(flow_web::update_tag).delete(flow_web::delete_tag))
        // 搜索路由
        .route("/api/v1alpha1/search", get(flow_web::search))
        // 主题管理路由
        .route("/api/v1alpha1/themes", get(flow_web::list_themes))
        .route("/api/v1alpha1/themes", axum::routing::post(flow_web::install_theme))
        .route("/api/v1alpha1/themes/:name", get(flow_web::get_theme))
        .route("/api/v1alpha1/themes/:name/activate", axum::routing::put(flow_web::activate_theme))
        .route("/api/v1alpha1/themes/:name/reload", axum::routing::post(flow_web::reload_theme))
        .route("/api/v1alpha1/themes/:name/upgrade", axum::routing::post(flow_web::upgrade_theme))
        // 主题静态资源路由
        .route("/themes/*path", get(flow_web::serve_theme_static))
        // 附件管理路由
        .route("/api/v1alpha1/attachments", get(flow_web::list_attachments).post(flow_web::upload_attachment))
        .route("/api/v1alpha1/attachments/:name", get(flow_web::get_attachment).put(flow_web::update_attachment).delete(flow_web::delete_attachment))
        .route("/api/v1alpha1/attachments/:name/thumbnails/:size", get(flow_web::get_thumbnail))
        // 共享URL路由
        .route("/api/v1alpha1/attachments/:name/shared-urls", get(flow_web::list_shared_urls).post(flow_web::generate_shared_url))
        .route("/api/v1alpha1/attachments/shared-urls/:token", axum::routing::delete(flow_web::revoke_shared_url))
        .route("/api/v1alpha1/attachments/shared/:token", get(flow_web::get_attachment_by_shared_url))
        // 通知管理路由
        .route("/api/v1alpha1/notifications", get(flow_web::list_notifications).post(flow_web::create_notification))
        .route("/api/v1alpha1/notifications/:name", get(flow_web::get_notification).put(flow_web::update_notification).delete(flow_web::delete_notification))
        .route("/api/v1alpha1/notifications/:name/read", axum::routing::put(flow_web::mark_notification_as_read))
        .route("/api/v1alpha1/notifications/read-all", axum::routing::put(flow_web::mark_all_notifications_as_read))
        .route("/api/v1alpha1/notifications/:recipient/unread-count", get(flow_web::get_unread_count))
        // 订阅管理路由
        .route("/api/v1alpha1/subscriptions", get(flow_web::list_subscriptions).post(flow_web::create_subscription))
        .route("/api/v1alpha1/subscriptions/:name", get(flow_web::get_subscription).delete(flow_web::delete_subscription))
        .route("/api/v1alpha1/subscriptions/:name/unsubscribe", get(flow_web::unsubscribe_by_token))
        // 原因管理路由
        .route("/api/v1alpha1/reasons", get(flow_web::list_reasons).post(flow_web::create_reason))
        .route("/api/v1alpha1/reasons/:name", get(flow_web::get_reason).delete(flow_web::delete_reason))
        // Policy管理路由
        .route("/api/v1alpha1/policies", get(flow_web::list_policies).post(flow_web::create_policy))
        .route("/api/v1alpha1/policies/:name", get(flow_web::get_policy).put(flow_web::update_policy).delete(flow_web::delete_policy))
        // Group管理路由
        .route("/api/v1alpha1/groups", get(flow_web::list_groups).post(flow_web::create_group))
        .route("/api/v1alpha1/groups/:name", get(flow_web::get_group).put(flow_web::update_group).delete(flow_web::delete_group))
        .route("/api/v1alpha1/groups/:name/update-count", axum::routing::post(flow_web::update_group_count))
        // UC端点（用户中心）
        .nest("/api/v1alpha1/uc", uc_routes())
        // Extension端点和WebSocket路由（共享/apis路径）
        // WebSocket路由需要放在Extension路由之前，因为Axum按顺序匹配路由
        .route("/apis/*path", axum::routing::get(flow_web::handle_websocket))
        .nest("/apis", extension_routes())
        // SwaggerUI文档 - 暂时注释掉，需要修复 utoipa-swagger-ui 9.0 的集成
        // .merge(SwaggerUi::new("/swagger-ui/*"))
        .layer(
            ServiceBuilder::new()
                // 注意：在Axum/Tower中，中间件的执行顺序与添加顺序相反
                // 最后一个添加的层会在请求时最先执行（最外层）
                // 第一个添加的层会在请求时最后执行（最内层，最接近handler）
                //
                // 我们想要的执行顺序（请求路径）：
                // rate_limit -> auth -> authorize -> handler
                //
                // 因此添加顺序应该是（从内到外）：
                // authorize -> auth -> rate_limit -> CORS
                
                // 授权中间件（最先添加，最后执行 - 最内层，需要认证中间件已经设置了用户信息）
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    |state: State<AppState>, request: Request<axum::body::Body>, next: Next| async move {
                        flow_web::authorize_middleware(state, request, next).await
                    },
                ))
                // 认证中间件（第二个添加，第二个执行 - 在授权之前执行以设置用户信息）
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    |state: State<AppState>, request: Request<axum::body::Body>, next: Next| async move {
                        flow_web::auth_middleware(state, request, next).await
                    },
                ))
                // 速率限制中间件（第三个添加，第一个执行 - 最外层，最先检查）
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    |state: State<AppState>, request: Request<axum::body::Body>, next: Next| async move {
                        flow_web::rate_limit_middleware(state, request, next).await
                    },
                ))
                // CORS中间件（最后添加，最外层执行）
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

/// UC路由（用户中心）
fn uc_routes() -> Router<AppState> {
    Router::new()
        .route("/posts", get(flow_web::list_my_posts).post(flow_web::create_my_post))
        .route("/posts/:name", get(flow_web::get_my_post).put(flow_web::update_my_post))
        .route("/posts/:name/publish", axum::routing::put(flow_web::publish_my_post))
        .route("/posts/:name/unpublish", axum::routing::put(flow_web::unpublish_my_post))
        .route("/posts/:name/recycle", axum::routing::delete(flow_web::recycle_my_post))
        .route("/posts/:name/draft", get(flow_web::get_my_post_draft).put(flow_web::update_my_post_draft))
}

/// Extension路由（动态路径）
/// 格式: /apis/{group}/{version}/{resource} 或 /apis/{group}/{version}/{resource}/{name}
/// 使用通配符路由匹配所有/apis/*路径
fn extension_routes() -> Router<AppState> {
    use axum::routing::{delete, get, patch, post, put};
    
    Router::new()
        // 使用通配符匹配所有路径
        // 格式: /apis/*path
        .route("/apis/*path", get(flow_web::handle_extension_get)
            .post(flow_web::handle_extension_post)
            .put(flow_web::handle_extension_put)
            .delete(flow_web::handle_extension_delete)
            .patch(flow_web::handle_extension_patch))
}

/// 健康检查端点
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// 初始化应用状态
pub async fn init_app_state(
    _db_manager: Arc<DatabaseManager>,
    jwt_service: Arc<JwtService>,
    session_service: Arc<dyn SessionService>,
    rate_limiter: Arc<dyn RateLimiter>,
    extension_client: Arc<ReactiveExtensionClient>,
    config: &crate::config::Config,
) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    // 创建服务层（使用具体类型，因为DefaultUserService和DefaultRoleService是泛型的）
    // 注意：由于trait object的限制，这里使用具体类型包装
    use flow_service::security::{
        user_service::DefaultUserService,
        role_service::DefaultRoleService,
        PasswordAlgorithm,
    };
    let user_service: Arc<dyn UserService> = Arc::new(
        DefaultUserService::new(extension_client.clone())
    );
    
    let role_service: Arc<dyn RoleService> = Arc::new(
        DefaultRoleService::new(extension_client.clone())
    );
    
    // 创建密码服务
    let password_service: Arc<dyn PasswordService> = Arc::new(
        DefaultPasswordService::new(PasswordAlgorithm::Bcrypt)
    );
    
    // 创建认证服务
    let auth_service = AuthService::new();
    
    // 创建PAT提供者
    let pat_provider = flow_web::PatProvider::new(
        extension_client.clone(),
        jwt_service.clone(),
    );
    auth_service.add_provider(Box::new(pat_provider));
    
    // 创建Basic Auth提供者
    let basic_auth_provider = flow_web::BasicAuthProvider::new(
        user_service.clone(),
        password_service.clone(),
        role_service.clone(),
    );
    auth_service.add_provider(Box::new(basic_auth_provider));
    
    // 创建Form Login提供者
    let form_login_provider = flow_web::FormLoginProvider::new(
        user_service.clone(),
        password_service.clone(),
        rate_limiter.clone(),
        session_service.clone(),
        jwt_service.clone(),
    );
    auth_service.add_provider(Box::new(form_login_provider));
    
    // TODO: 从配置中读取OAuth2配置
    // let oauth2_provider = flow_web::OAuth2Provider::new(...);
    // auth_service.add_provider(Box::new(oauth2_provider));
    
    let auth_service = Arc::new(auth_service);
    
    // 创建授权管理器
    let authorization_manager: Arc<dyn AuthorizationManager> = Arc::new(
        flow_service::security::DefaultAuthorizationManager::new(role_service.clone())
    );
    
    // 创建基础Post服务
    let base_post_service: Arc<dyn PostService> = Arc::new(
        DefaultPostService::new(extension_client.clone())
    );

    // 创建基础SinglePage服务
    let base_single_page_service: Arc<dyn SinglePageService> = Arc::new(
        DefaultSinglePageService::new(extension_client.clone())
    );

    // 创建Comment服务
    let comment_service: Arc<dyn CommentService> = Arc::new(
        DefaultCommentService::new(extension_client.clone())
    );

    // 创建Category服务
    let category_service: Arc<dyn CategoryService> = Arc::new(
        DefaultCategoryService::new(extension_client.clone())
    );

    // 创建Tag服务
    let tag_service: Arc<dyn TagService> = Arc::new(
        DefaultTagService::new(extension_client.clone())
    );

    // 初始化搜索服务
    let index_path = &config.flow.search.index_path;
    let search_engine: Arc<dyn SearchEngine> = Arc::new(
        TantivySearchEngine::new(index_path).await
            .map_err(|e| format!("Failed to initialize search engine: {}", e))?
    );
    let search_service: Arc<dyn SearchService> = Arc::new(
        DefaultSearchService::new(search_engine.clone())
    );

    // 初始化索引引擎
    let indices_manager = Arc::new(IndicesManager::new());
    let fulltext_mapping = Arc::new(FulltextFieldMapping::default());
    let index_engine = flow_infra::index::engine::DefaultIndexEngine::with_search_engine(
        indices_manager,
        Some(search_engine),
        fulltext_mapping,
    );

    // 创建带搜索索引的Post服务（包装基础服务）
    let post_service: Arc<dyn PostService> = Arc::new(
        SearchIndexingPostService::new(base_post_service.clone(), search_service.clone())
    );

    // 创建带搜索索引的SinglePage服务（包装基础服务）
    let single_page_service: Arc<dyn SinglePageService> = Arc::new(
        SearchIndexingSinglePageService::new(base_single_page_service.clone(), search_service.clone())
    );

    // 初始化附件服务
    use flow_service::attachment::{
        AttachmentService, DefaultAttachmentService,
        PolicyService, DefaultPolicyService,
        GroupService, DefaultGroupService,
        SharedUrlService, DefaultSharedUrlService,
    };
    use flow_service::attachment::thumbnail::{ThumbnailService, DefaultThumbnailService};
    use flow_infra::attachment::{AttachmentStorage, LocalAttachmentStorage};
    
    // 从配置中读取附件存储路径和基础URL
    let attachment_config = &config.flow.attachment;
    let attachment_root = if attachment_config.storage_path.is_absolute() {
        attachment_config.storage_path.clone()
    } else {
        config.flow.work_dir.join(&attachment_config.storage_path)
    };
    let thumbnail_dir = attachment_root.join("thumbnails");
    let upload_path = attachment_root.join("upload");
    
    // 确定基础URL
    let base_url = attachment_config.base_url.clone()
        .or_else(|| config.flow.external_url.clone())
        .unwrap_or_else(|| {
            format!("http://{}:{}", config.server.host, config.server.port)
        });
    
    // 创建存储服务
    let storage: Arc<dyn AttachmentStorage> = Arc::new(
        LocalAttachmentStorage::new(attachment_root.clone())
    );
    
    // 创建缩略图服务
    let thumbnail_service: Arc<dyn ThumbnailService> = Arc::new(
        DefaultThumbnailService::new(thumbnail_dir, attachment_config.thumbnail_quality)
    );
    
    // 创建附件服务
    let attachment_service: Arc<dyn AttachmentService> = Arc::new(
        DefaultAttachmentService::new(
            extension_client.clone(),
            storage,
            thumbnail_service,
            upload_path,
            base_url,
        )
    );
    
    // 创建Policy服务
    let policy_service: Arc<dyn PolicyService> = Arc::new(
        DefaultPolicyService::new(extension_client.clone())
    );
    
    // 创建Group服务
    let group_service: Arc<dyn GroupService> = Arc::new(
        DefaultGroupService::new(
            extension_client.clone(),
            attachment_service.clone(),
        )
    );
    
    // 创建共享URL服务
    let shared_url_service: Arc<dyn SharedUrlService> = Arc::new(
        DefaultSharedUrlService::new()
    );

    // 创建主题服务
    let theme_root = config.flow.work_dir.join("themes");
    let theme_service: Arc<dyn ThemeService> = Arc::new(
        DefaultThemeService::new(extension_client.clone(), theme_root.clone())
    );
    
    // 创建主题解析器和模板引擎管理器
    let theme_resolver = Arc::new(
        flow_infra::theme::ThemeResolver::new(
            extension_client.clone(),
            theme_root.clone()
        )
    );
    let template_engine_manager = Arc::new(
        flow_infra::theme::TemplateEngineManager::new(theme_root.clone())
    );
    
    // 创建WebSocket端点管理器
    let websocket_manager = Arc::new(
        flow_infra::websocket::WebSocketEndpointManager::new()
    );
    
    // 注册示例WebSocket端点（用于测试）
    use flow_infra::websocket::{WebSocketEndpoint, WebSocketEndpointManager};
    use flow_api::extension::GroupVersionKind;
    
    struct EchoEndpoint {
        group_version: GroupVersionKind,
        url_path: String,
    }
    
    impl WebSocketEndpoint for EchoEndpoint {
        fn url_path(&self) -> &str {
            &self.url_path
        }
        
        fn group_version(&self) -> GroupVersionKind {
            self.group_version.clone()
        }
    }
    
    let echo_endpoint = Arc::new(EchoEndpoint {
        group_version: GroupVersionKind::new("test.halo.run", "v1alpha1", "WebSocket"),
        url_path: "echo".to_string(),
    });
    
    websocket_manager.register(echo_endpoint).await;

    // 创建通知服务
    let notification_service: Arc<dyn NotificationService> = Arc::new(
        DefaultNotificationService::new(extension_client.clone())
    );
    
    // 创建一个简单的通知发送器实现（站内通知）
    struct InMemoryNotificationSender;
    
    #[async_trait]
    impl NotificationSender for InMemoryNotificationSender {
        async fn send_notification(
            &self,
            _notifier_extension_name: &str,
            _context: flow_service::notification::NotificationContext,
        ) -> anyhow::Result<()> {
            // TODO: 实现实际的通知发送逻辑
            // 目前站内通知已经通过NotificationService创建，这里可以用于扩展其他通知方式（邮件、短信等）
            Ok(())
        }
    }
    
    let notification_sender: Arc<dyn NotificationSender> = Arc::new(InMemoryNotificationSender);
    
    // 创建通知中心
    let notification_center: Arc<dyn NotificationCenter> = Arc::new(
        DefaultNotificationCenter::new(
            extension_client.clone(),
            notification_service.clone(),
            notification_sender,
        )
    );

    Ok(AppState {
        auth_service,
        authorization_manager,
        jwt_service,
        session_service,
        rate_limiter,
        extension_client,
        user_service,
        role_service,
        password_service,
        post_service,
        single_page_service,
        comment_service,
        category_service,
        tag_service,
        search_service,
        attachment_service,
        policy_service,
        group_service,
        shared_url_service,
        theme_service,
        theme_root,
        theme_resolver,
        template_engine_manager,
        websocket_manager,
        notification_service,
        notification_center,
    })
}

