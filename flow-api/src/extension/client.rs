use crate::extension::{Extension, ListOptions, ListResult};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// ExtensionClient trait 定义扩展对象的CRUD操作
#[async_trait]
pub trait ExtensionClient: Send + Sync {
    async fn create<E: Extension + Serialize>(&self, extension: E) -> Result<E, Box<dyn std::error::Error + Send + Sync>>;
    async fn update<E: Extension + Serialize>(&self, extension: E) -> Result<E, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete<E: Extension>(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn fetch<E: Extension + for<'de> Deserialize<'de>>(&self, name: &str) -> Result<Option<E>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list<E: Extension + for<'de> Deserialize<'de>>(&self, options: ListOptions) -> Result<ListResult<E>, Box<dyn std::error::Error + Send + Sync>>;
}

