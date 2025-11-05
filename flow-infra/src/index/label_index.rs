use flow_api::extension::Extension;
use flow_api::extension::index::{Index, LabelIndexQuery, TransactionalOperation};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// LabelEntry 表示一个标签条目（键值对）
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct LabelEntry {
    key: String,
    value: String,
}

impl LabelEntry {
    fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
    
    fn key_only(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: String::new(),
        }
    }
}

/// LabelIndex 实现标签索引
pub struct LabelIndex {
    /// 主索引: LabelEntry -> Set<primary_key>
    index: Arc<RwLock<std::collections::BTreeMap<LabelEntry, HashSet<String>>>>,
    /// 反向索引: primary_key -> Set<LabelEntry>
    inverted_index: Arc<RwLock<HashMap<String, HashSet<LabelEntry>>>>,
    /// 空标签集合: 没有标签的扩展对象的主键集合
    empty_labels_set: Arc<RwLock<HashSet<String>>>,
}

impl LabelIndex {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(std::collections::BTreeMap::new())),
            inverted_index: Arc::new(RwLock::new(HashMap::new())),
            empty_labels_set: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    /// 执行插入操作
    pub fn insert(&self, primary_key: &str, labels: Option<&HashMap<String, String>>) {
        // 删除旧索引
        self.remove(primary_key);
        
        if let Some(labels) = labels {
            if labels.is_empty() {
                self.empty_labels_set.write().unwrap().insert(primary_key.to_string());
            } else {
                let mut entry_set = HashSet::new();
                for (key, value) in labels {
                    let entry = LabelEntry::new(key, value);
                    self.index
                        .write()
                        .unwrap()
                        .entry(entry.clone())
                        .or_insert_with(HashSet::new)
                        .insert(primary_key.to_string());
                    entry_set.insert(entry);
                }
                self.inverted_index
                    .write()
                    .unwrap()
                    .insert(primary_key.to_string(), entry_set);
            }
        } else {
            self.empty_labels_set.write().unwrap().insert(primary_key.to_string());
        }
    }
    
    /// 执行删除操作
    pub fn remove(&self, primary_key: &str) {
        // 从反向索引中获取所有标签条目
        if let Some(entries) = self.inverted_index.write().unwrap().remove(primary_key) {
            // 从主索引中删除
            for entry in entries {
                if let Some(set) = self.index.write().unwrap().get_mut(&entry) {
                    set.remove(primary_key);
                    if set.is_empty() {
                        self.index.write().unwrap().remove(&entry);
                    }
                }
            }
        }
        self.empty_labels_set.write().unwrap().remove(primary_key);
    }
    
    /// 获取所有主键（用于NOT条件等）
    pub fn all_primary_keys(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        
        // 从反向索引获取所有主键
        result.extend(self.inverted_index.read().unwrap().keys().cloned());
        
        // 添加空标签集合中的主键
        result.extend(self.empty_labels_set.read().unwrap().iter().cloned());
        
        result
    }
}

impl Default for LabelIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Extension> Index<E, String> for LabelIndex {
    fn name(&self) -> &str {
        "metadata.labels"
    }
    
    fn key_type_name(&self) -> &str {
        "String"
    }
    
    fn is_unique(&self) -> bool {
        false
    }
    
    fn prepare_insert(&self, extension: &E) -> TransactionalOperation {
        let primary_key = extension.metadata().name.clone();
        let labels = extension.metadata().labels.as_ref();
        let keys = if let Some(labels) = labels {
            labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect()
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

impl LabelIndexQuery for LabelIndex {
    fn exists(&self, label_key: &str) -> HashSet<String> {
        let index = self.index.read().unwrap();
        let from = LabelEntry::key_only(label_key);
        let to = LabelEntry::new(label_key, char::MAX.to_string());
        
        let mut result = HashSet::new();
        for (entry, keys) in index.range(from..=to) {
            if entry.key == label_key {
                result.extend(keys.iter().cloned());
            }
        }
        result
    }
    
    fn equal(&self, label_key: &str, label_value: &str) -> HashSet<String> {
        let entry = LabelEntry::new(label_key, label_value);
        self.index
            .read()
            .unwrap()
            .get(&entry)
            .cloned()
            .unwrap_or_default()
    }
    
    fn not_equal(&self, label_key: &str, label_value: &str) -> HashSet<String> {
        let index = self.index.read().unwrap();
        let from = LabelEntry::key_only(label_key);
        let to = LabelEntry::new(label_key, char::MAX.to_string());
        
        let mut result = HashSet::new();
        for (entry, keys) in index.range(from..=to) {
            if entry.key == label_key && entry.value != label_value {
                result.extend(keys.iter().cloned());
            }
        }
        result
    }
    
    fn in_set(&self, label_key: &str, label_values: &[String]) -> HashSet<String> {
        let mut result = HashSet::new();
        for value in label_values {
            let entry = LabelEntry::new(label_key, value);
            if let Some(keys) = self.index.read().unwrap().get(&entry) {
                result.extend(keys.iter().cloned());
            }
        }
        result
    }
    
    fn not_in_set(&self, label_key: &str, label_values: &[String]) -> HashSet<String> {
        let index = self.index.read().unwrap();
        let from = LabelEntry::key_only(label_key);
        let to = LabelEntry::new(label_key, char::MAX.to_string());
        
        let value_set: HashSet<String> = label_values.iter().cloned().collect();
        let mut result = HashSet::new();
        for (entry, keys) in index.range(from..=to) {
            if entry.key == label_key && !value_set.contains(&entry.value) {
                result.extend(keys.iter().cloned());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_api::extension::{Extension, GroupVersionKind, Metadata};
    
    struct TestExtension {
        metadata: Metadata,
        gvk: GroupVersionKind,
    }
    
    impl Extension for TestExtension {
        fn metadata(&self) -> &Metadata {
            &self.metadata
        }
        
        fn group_version_kind(&self) -> GroupVersionKind {
            self.gvk.clone()
        }
    }
    
    #[test]
    fn test_label_index_insert() {
        let index = LabelIndex::new();
        let mut labels = HashMap::new();
        labels.insert("env".to_string(), "prod".to_string());
        labels.insert("app".to_string(), "test".to_string());
        
        index.insert("test-1", Some(&labels));
        
        let result = index.equal("env", "prod");
        assert!(result.contains("test-1"));
    }
    
    #[test]
    fn test_label_index_delete() {
        let index = LabelIndex::new();
        let mut labels = HashMap::new();
        labels.insert("env".to_string(), "prod".to_string());
        
        index.insert("test-1", Some(&labels));
        index.remove("test-1");
        
        let result = index.equal("env", "prod");
        assert!(!result.contains("test-1"));
    }
}

