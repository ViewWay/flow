use flow_api::extension::Extension;
use flow_api::extension::index::{Index, ValueIndexQuery, TransactionalOperation};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::cmp::Ordering;

/// SingleValueIndexSpec 定义单值索引规范
pub trait SingleValueIndexSpec<E: Extension, K: Clone + Ord + Send + Sync>: Send + Sync {
    fn name(&self) -> &str;
    fn get_value(&self, extension: &E) -> Option<K>;
    fn key_type_name(&self) -> &str;
    fn is_unique(&self) -> bool;
}

/// SingleValueIndex 实现单值索引
pub struct SingleValueIndex<E: Extension, K: Clone + Ord + Send + Sync> {
    /// 主索引: K -> Set<primary_key>
    index: Arc<RwLock<std::collections::BTreeMap<K, HashSet<String>>>>,
    /// 反向索引: primary_key -> K
    inverted_index: Arc<RwLock<std::collections::HashMap<String, K>>>,
    /// 空值集合: 值为None的扩展对象的主键集合
    null_key_values: Arc<RwLock<HashSet<String>>>,
    /// 索引规范
    spec: Box<dyn SingleValueIndexSpec<E, K> + Send + Sync>,
}

impl<E: Extension, K: Clone + Ord + Send + Sync> SingleValueIndex<E, K> {
    pub fn new(spec: Box<dyn SingleValueIndexSpec<E, K> + Send + Sync>) -> Self {
        Self {
            index: Arc::new(RwLock::new(std::collections::BTreeMap::new())),
            inverted_index: Arc::new(RwLock::new(std::collections::HashMap::new())),
            null_key_values: Arc::new(RwLock::new(HashSet::new())),
            spec,
        }
    }
    
    /// 执行插入操作
    pub fn insert(&self, primary_key: &str, value: Option<K>) {
        // 删除旧索引
        self.remove(primary_key);
        
        if let Some(key) = value {
            self.index
                .write()
                .unwrap()
                .entry(key.clone())
                .or_insert_with(HashSet::new)
                .insert(primary_key.to_string());
            self.inverted_index
                .write()
                .unwrap()
                .insert(primary_key.to_string(), key);
        } else {
            self.null_key_values.write().unwrap().insert(primary_key.to_string());
        }
    }
    
    /// 执行删除操作
    pub fn remove(&self, primary_key: &str) {
        if let Some(key) = self.inverted_index.write().unwrap().remove(primary_key) {
            if let Some(set) = self.index.write().unwrap().get_mut(&key) {
                set.remove(primary_key);
                if set.is_empty() {
                    self.index.write().unwrap().remove(&key);
                }
            }
        }
        self.null_key_values.write().unwrap().remove(primary_key);
    }
    
    /// 获取指定主键的索引键值
    pub fn get_key(&self, primary_key: &str) -> Option<K> {
        self.inverted_index.read().unwrap().get(primary_key).cloned()
    }
}

impl<E: Extension, K: Clone + Ord + Send + Sync> Index<E, K> for SingleValueIndex<E, K> {
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
        let value = self.spec.get_value(extension);
        let keys = if let Some(_v) = value {
            vec!["value".to_string()]
        } else {
            Vec::new()
        };
        TransactionalOperation::upsert(primary_key, keys)
    }
    
    fn prepare_update(&self, extension: &E) -> TransactionalOperation {
        self.prepare_insert(extension)
    }
    
    fn prepare_delete(&self, primary_key: &str) -> TransactionalOperation {
        TransactionalOperation::delete(primary_key)
    }
}

impl<E: Extension, K: Clone + Ord + Send + Sync> ValueIndexQuery<K> for SingleValueIndex<E, K> {
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

