use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::get,
    Router,
};
use flow_api::security::AuthorizationManager;
use flow_service::security::{
    AuthService, RoleService, UserService, PasswordService, DefaultPasswordService,
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
        // TODO: 添加更多路由
        // .nest("/api/v1alpha1", api_routes())
        // .nest("/apis", extension_routes())
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
    
    Ok(AppState {
        auth_service,
        authorization_manager,
        jwt_service,
        session_service,
        rate_limiter,
        extension_client,
        user_service,
        role_service,
    })
}

