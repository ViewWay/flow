use sea_orm::{Database, DatabaseConnection, DbErr};
use redis::Client as RedisClient;
use mongodb::Client as MongoClient;
use std::sync::Arc;

/// DatabaseManager 管理所有数据库连接
#[derive(Clone)]
pub struct DatabaseManager {
    mysql: Option<Arc<DatabaseConnection>>,
    postgresql: Option<Arc<DatabaseConnection>>,
    redis: Option<Arc<RedisClient>>,
    mongodb: Option<Arc<MongoClient>>,
}

impl DatabaseManager {
    /// 创建新的DatabaseManager
    pub async fn new(
        mysql_url: Option<&str>,
        postgresql_url: Option<&str>,
        redis_url: Option<&str>,
        mongodb_url: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut manager = Self {
            mysql: None,
            postgresql: None,
            redis: None,
            mongodb: None,
        };

        // 连接MySQL
        if let Some(url) = mysql_url {
            let db = Database::connect(url).await?;
            manager.mysql = Some(Arc::new(db));
        }

        // 连接PostgreSQL
        if let Some(url) = postgresql_url {
            let db = Database::connect(url).await?;
            manager.postgresql = Some(Arc::new(db));
        }

        // 连接Redis
        if let Some(url) = redis_url {
            let client = RedisClient::open(url)?;
            manager.redis = Some(Arc::new(client));
        }

        // 连接MongoDB
        if let Some(url) = mongodb_url {
            let client = MongoClient::with_uri_str(url).await?;
            manager.mongodb = Some(Arc::new(client));
        }

        Ok(manager)
    }

    /// 获取MySQL连接
    pub fn mysql(&self) -> Option<Arc<DatabaseConnection>> {
        self.mysql.clone()
    }

    /// 获取PostgreSQL连接
    pub fn postgresql(&self) -> Option<Arc<DatabaseConnection>> {
        self.postgresql.clone()
    }

    /// 获取主数据库连接（优先PostgreSQL，其次MySQL）
    pub fn primary_db(&self) -> Result<Arc<DatabaseConnection>, DbErr> {
        self.postgresql
            .clone()
            .or_else(|| self.mysql.clone())
            .ok_or_else(|| DbErr::Custom("No database connection available".to_string()))
    }

    /// 获取Redis连接
    pub fn redis(&self) -> Option<Arc<RedisClient>> {
        self.redis.clone()
    }

    /// 获取MongoDB连接
    pub fn mongodb(&self) -> Option<Arc<MongoClient>> {
        self.mongodb.clone()
    }
}

