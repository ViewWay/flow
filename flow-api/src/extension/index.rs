use crate::extension::Extension;
use std::collections::HashSet;

/// Index trait 定义索引的基本操作
pub trait Index<E: Extension, K: Clone + Ord + Send + Sync>: Send + Sync {
    /// 获取索引名称
    fn name(&self) -> &str;
    
    /// 获取索引键类型名称
    fn key_type_name(&self) -> &str;
    
    /// 索引是否唯一
    fn is_unique(&self) -> bool {
        false
    }
    
    /// 准备插入操作
    fn prepare_insert(&self, extension: &E) -> TransactionalOperation;
    
    /// 准备更新操作
    fn prepare_update(&self, extension: &E) -> TransactionalOperation;
    
    /// 准备删除操作
    fn prepare_delete(&self, primary_key: &str) -> TransactionalOperation;
}

/// TransactionalOperation 表示一个事务性操作
#[derive(Debug, Clone)]
pub enum TransactionalOperation {
    Upsert {
        primary_key: String,
        keys: Vec<String>,
    },
    Delete {
        primary_key: String,
    },
}

impl TransactionalOperation {
    pub fn upsert(primary_key: impl Into<String>, keys: Vec<String>) -> Self {
        Self::Upsert {
            primary_key: primary_key.into(),
            keys,
        }
    }
    
    pub fn delete(primary_key: impl Into<String>) -> Self {
        Self::Delete {
            primary_key: primary_key.into(),
        }
    }
}

/// ValueIndexQuery trait 定义值索引的查询操作
pub trait ValueIndexQuery<K: Clone + Ord + Send + Sync>: Send + Sync {
    /// 等于查询
    fn equal(&self, key: &K) -> HashSet<String>;
    
    /// 不等于查询
    fn not_equal(&self, key: &K) -> HashSet<String>;
    
    /// 范围查询
    fn between(
        &self,
        from_key: &K,
        from_inclusive: bool,
        to_key: &K,
        to_inclusive: bool,
    ) -> HashSet<String>;
    
    /// 不在范围内查询
    fn not_between(
        &self,
        from_key: &K,
        from_inclusive: bool,
        to_key: &K,
        to_inclusive: bool,
    ) -> HashSet<String>;
    
    /// 大于查询
    fn greater_than(&self, key: &K, inclusive: bool) -> HashSet<String>;
    
    /// 小于查询
    fn less_than(&self, key: &K, inclusive: bool) -> HashSet<String>;
    
    /// 获取所有主键
    fn all(&self) -> HashSet<String>;
}

/// LabelIndexQuery trait 定义标签索引的查询操作
pub trait LabelIndexQuery: Send + Sync {
    /// 检查标签键是否存在
    fn exists(&self, label_key: &str) -> HashSet<String>;
    
    /// 等于查询
    fn equal(&self, label_key: &str, label_value: &str) -> HashSet<String>;
    
    /// 不等于查询
    fn not_equal(&self, label_key: &str, label_value: &str) -> HashSet<String>;
    
    /// 在集合中查询
    fn in_set(&self, label_key: &str, label_values: &[String]) -> HashSet<String>;
    
    /// 不在集合中查询
    fn not_in_set(&self, label_key: &str, label_values: &[String]) -> HashSet<String>;
}

/// IndexSpec trait 定义索引规范
pub trait IndexSpec<E: Extension>: Send + Sync {
    fn name(&self) -> &str;
    fn get_values(&self, extension: &E) -> Vec<String>;
}

