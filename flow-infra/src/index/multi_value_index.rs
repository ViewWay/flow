use flow_api::extension::Extension;
use flow_api::extension::index::{Index, ValueIndexQuery, TransactionalOperation};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::cmp::Ordering;

/// MultiValueIndexSpec 定义多值索引规范
pub trait MultiValueIndexSpec<E: Extension, K: Clone + Ord + Send + Sync + std::hash::Hash>: Send + Sync {
    fn name(&self) -> &str;
    fn get_values(&self, extension: &E) -> Vec<K>;
    fn key_type_name(&self) -> &str;
    fn is_unique(&self) -> bool;
}

/// MultiValueIndex 实现多值索引
pub struct MultiValueIndex<E: Extension, K: Clone + Ord + Send + Sync + std::hash::Hash> {
    /// 主索引: K -> Set<primary_key>
    index: Arc<RwLock<std::collections::BTreeMap<K, HashSet<String>>>>,
    /// 反向索引: primary_key -> Set<K>
    inverted_index: Arc<RwLock<std::collections::HashMap<String, HashSet<K>>>>,
    /// 空值集合: 值为空的扩展对象的主键集合
    null_key_values: Arc<RwLock<HashSet<String>>>,
    /// 索引规范
    spec: Box<dyn MultiValueIndexSpec<E, K> + Send + Sync>,
}

impl<E: Extension, K: Clone + Ord + Send + Sync + std::hash::Hash> MultiValueIndex<E, K> {
    pub fn new(spec: Box<dyn MultiValueIndexSpec<E, K> + Send + Sync>) -> Self {
        Self {
            index: Arc::new(RwLock::new(std::collections::BTreeMap::new())),
            inverted_index: Arc::new(RwLock::new(std::collections::HashMap::new())),
            null_key_values: Arc::new(RwLock::new(HashSet::new())),
            spec,
        }
    }
    
    /// 执行插入操作
    pub fn insert(&self, primary_key: &str, values: Vec<K>) {
        // 删除旧索引
        self.remove(primary_key);
        
        if values.is_empty() {
            self.null_key_values.write().unwrap().insert(primary_key.to_string());
        } else {
            let mut key_set = HashSet::new();
            let mut index_guard = self.index.write().unwrap();
            for key in values {
                index_guard
                    .entry(key.clone())
                    .or_insert_with(HashSet::new)
                    .insert(primary_key.to_string());
                key_set.insert(key);
            }
            drop(index_guard); // 释放锁
            self.inverted_index
                .write()
                .unwrap()
                .insert(primary_key.to_string(), key_set);
        }
    }
    
    /// 执行删除操作
    pub fn remove(&self, primary_key: &str) {
        if let Some(keys) = self.inverted_index.write().unwrap().remove(primary_key) {
            for key in keys {
                if let Some(set) = self.index.write().unwrap().get_mut(&key) {
                    set.remove(primary_key);
                    if set.is_empty() {
                        self.index.write().unwrap().remove(&key);
                    }
                }
            }
        }
        self.null_key_values.write().unwrap().remove(primary_key);
    }
    
    /// 获取指定主键的所有索引键值
    pub fn get_keys(&self, primary_key: &str) -> HashSet<K> {
        self.inverted_index
            .read()
            .unwrap()
            .get(primary_key)
            .cloned()
            .unwrap_or_default()
    }
}

impl<E: Extension, K: Clone + Ord + Send + Sync + std::hash::Hash> Index<E, K> for MultiValueIndex<E, K> {
    fn name(&self) -> &str {
        self.spec.name()
    }
    
    fn key_type_name(&self) -> &str {
        self.spec.key_type_name()
    }
    
    fn is_unique(&self) -> bool {
        self.spec.is_unique()
    }
    
    fn prepare_insert(&self, extension: &E) -> TransactionalOperation {
        let primary_key = extension.metadata().name.clone();
        let values = self.spec.get_values(extension);
        let keys: Vec<String> = values.iter().map(|_v| "value".to_string()).collect();
        TransactionalOperation::upsert(primary_key, keys)
    }
    
    fn prepare_update(&self, extension: &E) -> TransactionalOperation {
        self.prepare_insert(extension)
    }
    
    fn prepare_delete(&self, primary_key: &str) -> TransactionalOperation {
        TransactionalOperation::delete(primary_key)
    }
}

impl<E: Extension, K: Clone + Ord + Send + Sync + std::hash::Hash> ValueIndexQuery<K> for MultiValueIndex<E, K> {
    fn equal(&self, key: &K) -> HashSet<String> {
        self.index
            .read()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }
    
    fn not_equal(&self, key: &K) -> HashSet<String> {
        let index = self.index.read().unwrap();
        let mut result = HashSet::new();
        for (k, keys) in index.iter() {
            if k != key {
                result.extend(keys.iter().cloned());
            }
        }
        result.extend(self.null_key_values.read().unwrap().iter().cloned());
        result
    }
    
    fn between(
        &self,
        from_key: &K,
        from_inclusive: bool,
        to_key: &K,
        to_inclusive: bool,
    ) -> HashSet<String> {
        let index = self.index.read().unwrap();
        let mut result = HashSet::new();
        
        for (key, keys) in index.iter() {
            let cmp_from = key.cmp(from_key);
            let cmp_to = key.cmp(to_key);
            
            let matches_from = if from_inclusive {
                matches!(cmp_from, Ordering::Greater | Ordering::Equal)
            } else {
                matches!(cmp_from, Ordering::Greater)
            };
            
            let matches_to = if to_inclusive {
                matches!(cmp_to, Ordering::Less | Ordering::Equal)
            } else {
                matches!(cmp_to, Ordering::Less)
            };
            
            if matches_from && matches_to {
                result.extend(keys.iter().cloned());
            }
        }
        
        result
    }
    
    fn not_between(
        &self,
        from_key: &K,
        from_inclusive: bool,
        to_key: &K,
        to_inclusive: bool,
    ) -> HashSet<String> {
        let all = self.all();
        let between = self.between(from_key, from_inclusive, to_key, to_inclusive);
        all.difference(&between).cloned().collect()
    }
    
    fn greater_than(&self, key: &K, inclusive: bool) -> HashSet<String> {
        let index = self.index.read().unwrap();
        let mut result = HashSet::new();
        
        for (k, keys) in index.iter() {
            let cmp = k.cmp(key);
            let matches = if inclusive {
                matches!(cmp, Ordering::Greater | Ordering::Equal)
            } else {
                matches!(cmp, Ordering::Greater)
            };
            if matches {
                result.extend(keys.iter().cloned());
            }
        }
        
        result
    }
    
    fn less_than(&self, key: &K, inclusive: bool) -> HashSet<String> {
        let index = self.index.read().unwrap();
        let mut result = HashSet::new();
        
        for (k, keys) in index.iter() {
            let cmp = k.cmp(key);
            let matches = if inclusive {
                matches!(cmp, Ordering::Less | Ordering::Equal)
            } else {
                matches!(cmp, Ordering::Less)
            };
            if matches {
                result.extend(keys.iter().cloned());
            }
        }
        
        result
    }
    
    fn all(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        for keys in self.index.read().unwrap().values() {
            result.extend(keys.iter().cloned());
        }
        result.extend(self.null_key_values.read().unwrap().iter().cloned());
        result
    }
}

