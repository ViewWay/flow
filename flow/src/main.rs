mod config;
mod error;
mod server;

use config::Config;
use error::Result;
use flow_infra::{
    database::DatabaseManager,
    cache::{Cache, RedisCache},
    security::{JwtService, SessionService, RateLimiter, RedisSessionService, RedisRateLimiter},
    extension::ReactiveExtensionClient,
    database::repository::SeaOrmExtensionRepository,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use axum::serve;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting Flow application...");

    // 加载配置
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // 初始化数据库连接
    let mysql_url = config.database.mysql.as_ref().map(|c| c.url.as_str());
    let postgresql_url = config.database.postgresql.as_ref().map(|c| c.url.as_str());
    let redis_url = Some(config.redis.url.as_str());
    let mongodb_url = Some(config.mongodb.url.as_str());

    let db_manager = Arc::new(
        DatabaseManager::new(mysql_url, postgresql_url, redis_url, mongodb_url).await?
    );
    info!("Database connections established");

    // 获取主数据库连接
    let primary_db = db_manager.primary_db()?;
    
    // 创建ExtensionRepository和ExtensionClient
    let repository: Arc<dyn flow_infra::database::repository::ExtensionRepository> = 
        Arc::new(SeaOrmExtensionRepository::new(primary_db.clone()));
    let extension_client = Arc::new(ReactiveExtensionClient::new(repository.clone()));

    // 初始化Redis缓存
    let redis_client = db_manager.redis()
        .ok_or("Redis connection not available")?;
    let cache: Arc<dyn Cache> = Arc::new(RedisCache::new(redis_client));

    // 初始化JWT服务
    let jwt_service = Arc::new(JwtService::new(
        &config.flow.security.jwt_secret,
        "flow".to_string(),
        config.flow.security.jwt_expiration,
    )?);

    // 初始化Session服务
    let session_service: Arc<dyn SessionService> = Arc::new(
        RedisSessionService::new(cache.clone(), 3600)
    );

    // 初始化速率限制器
    let redis_client_for_rate_limit = db_manager.redis()
        .ok_or("Redis connection not available")?;
    let rate_limiter: Arc<dyn RateLimiter> = Arc::new(
        RedisRateLimiter::new(redis_client_for_rate_limit)
    );

    // 初始化应用状态
    let app_state = server::init_app_state(
        db_manager,
        jwt_service,
        session_service,
        rate_limiter,
        extension_client,
    ).await?;
    info!("Application state initialized");

    // 创建路由
    let app = server::create_router(app_state);
    info!("Router created");

    // 启动HTTP服务器
    let addr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    
    info!("Flow application started successfully");
    info!("Server listening on {}", addr);

    let listener = TcpListener::bind(&addr).await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;
    
    serve(listener, app.into_make_service())
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
