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
    PostService, DefaultPostService,
    SinglePageService, DefaultSinglePageService,
    CommentService, DefaultCommentService,
    CategoryService, DefaultCategoryService,
    TagService, DefaultTagService,
};
use flow_infra::{
    database::DatabaseManager,
    security::{JwtService, SessionService, RateLimiter},
    extension::ReactiveExtensionClient,
};
use flow_web::AppState;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

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
        // UC端点（用户中心）
        .nest("/api/v1alpha1/uc", uc_routes())
        // Extension端点（动态路径）
        .nest("/apis", extension_routes())
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
    
    // 创建Post服务
    let post_service: Arc<dyn PostService> = Arc::new(
        DefaultPostService::new(extension_client.clone())
    );

    // 创建SinglePage服务
    let single_page_service: Arc<dyn SinglePageService> = Arc::new(
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
    })
}

