use async_trait::async_trait;
use mongodb::Client as MongoClient;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};

/// LogEntry 表示一条日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub target: String,
    pub fields: Option<serde_json::Value>,
}

/// Logger trait 定义日志操作
#[async_trait]
pub trait Logger: Send + Sync {
    async fn log(&self, entry: LogEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// MongoLogger 使用MongoDB实现的日志记录器
pub struct MongoLogger {
    collection: Collection<LogEntry>,
}

impl MongoLogger {
    pub fn new(client: Arc<MongoClient>, database: &str, collection: &str) -> Self {
        let db = client.database(database);
        let collection = db.collection(collection);
        Self { collection }
    }
}

#[async_trait]
impl Logger for MongoLogger {
    async fn log(&self, entry: LogEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.collection
            .insert_one(entry)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok(())
    }
}

