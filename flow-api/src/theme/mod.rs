/// Finder系统相关类型定义
/// Finder是模板中可以调用的数据查询接口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Finder trait - 所有Finder必须实现此trait
#[async_trait]
pub trait Finder: Send + Sync {
    /// Finder的名称（在模板中使用）
    fn name(&self) -> &str;
}

/// Finder注册表
pub trait FinderRegistry: Send + Sync {
    /// 注册Finder
    fn register(&self, name: String, finder: Box<dyn Finder>);
    
    /// 获取Finder
    fn get(&self, name: &str) -> Option<&dyn Finder>;
    
    /// 获取所有Finder
    fn get_all(&self) -> std::collections::HashMap<String, Box<dyn Finder>>;
    
    /// 移除Finder
    fn remove(&self, name: &str);
}

