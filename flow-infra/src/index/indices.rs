use flow_api::extension::Extension;
use flow_api::extension::index::{Index, ValueIndexQuery};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::label_index::LabelIndex;
use super::single_value_index::SingleValueIndex;
use super::multi_value_index::MultiValueIndex;

/// Indices 管理某个扩展类型的所有索引
pub struct Indices<E: Extension + 'static> {
    /// 所有索引的映射: index_name -> Index
    indices: Arc<RwLock<HashMap<String, Box<dyn AnyIndex<E> + Send + Sync>>>>,
    /// 标签索引（总是存在）
    label_index: Arc<LabelIndex>,
}

/// AnyIndex trait 用于类型擦除，提供查询接口
trait AnyIndex<E: Extension>: Send + Sync {
    fn name(&self) -> &str;
    fn key_type_name(&self) -> &str;
    
    /// 执行等于查询（接受JSON值，内部进行类型转换）
    fn query_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String>;
    
    /// 执行不等于查询
    fn query_not_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String>;
    
    /// 执行IN查询
    fn query_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String>;
    
    /// 执行NOT IN查询
    fn query_not_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String>;
    
    /// 执行小于查询
    fn query_less_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String>;
    
    /// 执行大于查询
    fn query_greater_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String>;
    
    /// 执行BETWEEN查询
    fn query_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String>;
    
    /// 执行NOT BETWEEN查询
    fn query_not_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String>;
    
    /// 获取所有主键
    fn query_all(&self) -> HashSet<String>;
    
    /// 执行字符串包含查询（仅对String类型索引有效）
    fn query_string_contains(&self, keyword: &str) -> Result<HashSet<String>, String>;
}

// 为SingleValueIndex实现AnyIndex（支持String类型）
impl<E: Extension + 'static> AnyIndex<E> for SingleValueIndex<E, String> {
    fn name(&self) -> &str {
        <SingleValueIndex<E, String> as Index<E, String>>::name(self)
    }
    
    fn key_type_name(&self) -> &str {
        <SingleValueIndex<E, String> as Index<E, String>>::key_type_name(self)
    }
    
    fn query_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(value.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::equal(self, &key))
    }
    
    fn query_not_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(value.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::not_equal(self, &key))
    }
    
    fn query_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let mut result = HashSet::new();
        for value in values {
            let key: String = serde_json::from_value(value.clone())
                .map_err(|e| format!("Cannot convert to String: {}", e))?;
            result.extend(ValueIndexQuery::<String>::equal(self, &key));
        }
        Ok(result)
    }
    
    fn query_not_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let all = self.query_all();
        let matched = self.query_in(values)?;
        Ok(all.difference(&matched).cloned().collect())
    }
    
    fn query_less_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(bound.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::less_than(self, &key, inclusive))
    }
    
    fn query_greater_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(bound.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::greater_than(self, &key, inclusive))
    }
    
    fn query_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let from: String = serde_json::from_value(from_key.clone())
            .map_err(|e| format!("Cannot convert from_key to String: {}", e))?;
        let to: String = serde_json::from_value(to_key.clone())
            .map_err(|e| format!("Cannot convert to_key to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::between(self, &from, from_inclusive, &to, to_inclusive))
    }
    
    fn query_not_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let from: String = serde_json::from_value(from_key.clone())
            .map_err(|e| format!("Cannot convert from_key to String: {}", e))?;
        let to: String = serde_json::from_value(to_key.clone())
            .map_err(|e| format!("Cannot convert to_key to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::not_between(self, &from, from_inclusive, &to, to_inclusive))
    }
    
    fn query_all(&self) -> HashSet<String> {
        ValueIndexQuery::<String>::all(self)
    }
    
    fn query_string_contains(&self, keyword: &str) -> Result<HashSet<String>, String> {
        // 获取所有键值对，然后过滤包含关键词的
        let all_keys = ValueIndexQuery::<String>::all(self);
        let keyword_lower = keyword.to_lowercase();
        let mut result = HashSet::new();
        
        // 遍历所有主键，检查其对应的索引值是否包含关键词
        for primary_key in &all_keys {
            if let Some(key_value) = self.get_key(primary_key) {
                if key_value.to_lowercase().contains(&keyword_lower) {
                    result.insert(primary_key.clone());
                }
            }
        }
        
        Ok(result)
    }
}

// 为SingleValueIndex实现AnyIndex（支持i64类型）
impl<E: Extension + 'static> AnyIndex<E> for SingleValueIndex<E, i64> {
    fn name(&self) -> &str {
        <SingleValueIndex<E, i64> as Index<E, i64>>::name(self)
    }
    
    fn key_type_name(&self) -> &str {
        <SingleValueIndex<E, i64> as Index<E, i64>>::key_type_name(self)
    }
    
    fn query_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let key: i64 = serde_json::from_value(value.clone())
            .map_err(|e| format!("Cannot convert to i64: {}", e))?;
        Ok(ValueIndexQuery::<i64>::equal(self, &key))
    }
    
    fn query_not_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let key: i64 = serde_json::from_value(value.clone())
            .map_err(|e| format!("Cannot convert to i64: {}", e))?;
        Ok(ValueIndexQuery::<i64>::not_equal(self, &key))
    }
    
    fn query_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let mut result = HashSet::new();
        for value in values {
            let key: i64 = serde_json::from_value(value.clone())
                .map_err(|e| format!("Cannot convert to i64: {}", e))?;
            result.extend(ValueIndexQuery::<i64>::equal(self, &key));
        }
        Ok(result)
    }
    
    fn query_not_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let all = self.query_all();
        let matched = self.query_in(values)?;
        Ok(all.difference(&matched).cloned().collect())
    }
    
    fn query_less_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let key: i64 = serde_json::from_value(bound.clone())
            .map_err(|e| format!("Cannot convert to i64: {}", e))?;
        Ok(ValueIndexQuery::<i64>::less_than(self, &key, inclusive))
    }
    
    fn query_greater_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let key: i64 = serde_json::from_value(bound.clone())
            .map_err(|e| format!("Cannot convert to i64: {}", e))?;
        Ok(ValueIndexQuery::<i64>::greater_than(self, &key, inclusive))
    }
    
    fn query_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let from: i64 = serde_json::from_value(from_key.clone())
            .map_err(|e| format!("Cannot convert from_key to i64: {}", e))?;
        let to: i64 = serde_json::from_value(to_key.clone())
            .map_err(|e| format!("Cannot convert to_key to i64: {}", e))?;
        Ok(ValueIndexQuery::<i64>::between(self, &from, from_inclusive, &to, to_inclusive))
    }
    
    fn query_not_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let from: i64 = serde_json::from_value(from_key.clone())
            .map_err(|e| format!("Cannot convert from_key to i64: {}", e))?;
        let to: i64 = serde_json::from_value(to_key.clone())
            .map_err(|e| format!("Cannot convert to_key to i64: {}", e))?;
        Ok(ValueIndexQuery::<i64>::not_between(self, &from, from_inclusive, &to, to_inclusive))
    }
    
    fn query_all(&self) -> HashSet<String> {
        ValueIndexQuery::<i64>::all(self)
    }
    
    fn query_string_contains(&self, _keyword: &str) -> Result<HashSet<String>, String> {
        Err("String contains query is only supported for String type indices".to_string())
    }
}

// 为MultiValueIndex实现AnyIndex（支持String类型）
impl<E: Extension + 'static> AnyIndex<E> for MultiValueIndex<E, String> {
    fn name(&self) -> &str {
        <MultiValueIndex<E, String> as Index<E, String>>::name(self)
    }
    
    fn key_type_name(&self) -> &str {
        <MultiValueIndex<E, String> as Index<E, String>>::key_type_name(self)
    }
    
    fn query_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(value.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::equal(self, &key))
    }
    
    fn query_not_equal(&self, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(value.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::not_equal(self, &key))
    }
    
    fn query_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let mut result = HashSet::new();
        for value in values {
            let key: String = serde_json::from_value(value.clone())
                .map_err(|e| format!("Cannot convert to String: {}", e))?;
            result.extend(ValueIndexQuery::<String>::equal(self, &key));
        }
        Ok(result)
    }
    
    fn query_not_in(&self, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let all = self.query_all();
        let matched = self.query_in(values)?;
        Ok(all.difference(&matched).cloned().collect())
    }
    
    fn query_less_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(bound.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::less_than(self, &key, inclusive))
    }
    
    fn query_greater_than(&self, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let key: String = serde_json::from_value(bound.clone())
            .map_err(|e| format!("Cannot convert to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::greater_than(self, &key, inclusive))
    }
    
    fn query_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let from: String = serde_json::from_value(from_key.clone())
            .map_err(|e| format!("Cannot convert from_key to String: {}", e))?;
        let to: String = serde_json::from_value(to_key.clone())
            .map_err(|e| format!("Cannot convert to_key to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::between(self, &from, from_inclusive, &to, to_inclusive))
    }
    
    fn query_not_between(
        &self,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let from: String = serde_json::from_value(from_key.clone())
            .map_err(|e| format!("Cannot convert from_key to String: {}", e))?;
        let to: String = serde_json::from_value(to_key.clone())
            .map_err(|e| format!("Cannot convert to_key to String: {}", e))?;
        Ok(ValueIndexQuery::<String>::not_between(self, &from, from_inclusive, &to, to_inclusive))
    }
    
    fn query_all(&self) -> HashSet<String> {
        ValueIndexQuery::<String>::all(self)
    }
    
    fn query_string_contains(&self, keyword: &str) -> Result<HashSet<String>, String> {
        // 对于多值索引，检查所有值是否包含关键词
        let all_keys = ValueIndexQuery::<String>::all(self);
        let keyword_lower = keyword.to_lowercase();
        let mut result = HashSet::new();
        
        // 遍历所有主键，检查其对应的所有索引值是否包含关键词
        for primary_key in &all_keys {
            let key_values = self.get_keys(primary_key);
            for key_value in &key_values {
                if key_value.to_lowercase().contains(&keyword_lower) {
                    result.insert(primary_key.clone());
                    break; // 找到一个匹配即可
                }
            }
        }
        
        Ok(result)
    }
}

impl<E: Extension + 'static> Indices<E> {
    pub fn new() -> Self {
        Self {
            indices: Arc::new(RwLock::new(HashMap::new())),
            label_index: Arc::new(LabelIndex::new()),
        }
    }
    
    /// 添加字符串类型的单值索引
    pub fn add_string_index(&self, index: SingleValueIndex<E, String>) {
        let name = <SingleValueIndex<E, String> as Index<E, String>>::name(&index);
        self.indices
            .write()
            .unwrap()
            .insert(name.to_string(), Box::new(index));
    }
    
    /// 添加i64类型的单值索引
    pub fn add_i64_index(&self, index: SingleValueIndex<E, i64>) {
        let name = <SingleValueIndex<E, i64> as Index<E, i64>>::name(&index);
        self.indices
            .write()
            .unwrap()
            .insert(name.to_string(), Box::new(index));
    }
    
    /// 添加字符串类型的多值索引
    pub fn add_string_multi_index(&self, index: MultiValueIndex<E, String>) {
        let name = <MultiValueIndex<E, String> as Index<E, String>>::name(&index);
        self.indices
            .write()
            .unwrap()
            .insert(name.to_string(), Box::new(index));
    }
    
    /// 获取索引名称列表
    pub fn get_index_names(&self) -> Vec<String> {
        self.indices.read().unwrap().keys().cloned().collect()
    }
    
    /// 执行等于查询（通过索引名称）
    pub fn query_equal(&self, index_name: &str, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_equal(value)
    }
    
    /// 执行不等于查询
    pub fn query_not_equal(&self, index_name: &str, value: &serde_json::Value) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_not_equal(value)
    }
    
    /// 执行IN查询
    pub fn query_in(&self, index_name: &str, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_in(values)
    }
    
    /// 执行NOT IN查询
    pub fn query_not_in(&self, index_name: &str, values: &[serde_json::Value]) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_not_in(values)
    }
    
    /// 执行小于查询
    pub fn query_less_than(&self, index_name: &str, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_less_than(bound, inclusive)
    }
    
    /// 执行大于查询
    pub fn query_greater_than(&self, index_name: &str, bound: &serde_json::Value, inclusive: bool) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_greater_than(bound, inclusive)
    }
    
    /// 执行BETWEEN查询
    pub fn query_between(
        &self,
        index_name: &str,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_between(from_key, from_inclusive, to_key, to_inclusive)
    }
    
    /// 执行NOT BETWEEN查询
    pub fn query_not_between(
        &self,
        index_name: &str,
        from_key: &serde_json::Value,
        from_inclusive: bool,
        to_key: &serde_json::Value,
        to_inclusive: bool,
    ) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_not_between(from_key, from_inclusive, to_key, to_inclusive)
    }
    
    /// 获取所有主键（通过索引名称）
    pub fn query_all(&self, index_name: &str) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        Ok(any_index.query_all())
    }
    
    /// 执行字符串包含查询（通过索引名称）
    pub fn query_string_contains(&self, index_name: &str, keyword: &str) -> Result<HashSet<String>, String> {
        let indices = self.indices.read().unwrap();
        let any_index = indices.get(index_name)
            .ok_or_else(|| format!("Index not found: {}", index_name))?;
        any_index.query_string_contains(keyword)
    }
    
    /// 插入扩展对象
    pub fn insert(&self, extension: &E) {
        // 更新标签索引
        let metadata = extension.metadata();
        self.label_index.insert(
            &metadata.name,
            metadata.labels.as_ref(),
        );
        
        // 更新其他索引
        // TODO: 需要根据索引规范更新索引
    }
    
    /// 更新扩展对象
    pub fn update(&self, extension: &E) {
        self.insert(extension);
    }
    
    /// 删除扩展对象
    pub fn delete(&self, extension: &E) {
        let primary_key = &extension.metadata().name;
        self.label_index.remove(primary_key);
        // TODO: 从其他索引中删除
    }
    
    /// 获取标签索引
    pub fn label_index(&self) -> Arc<LabelIndex> {
        Arc::clone(&self.label_index)
    }
}

impl<E: Extension + 'static> Default for Indices<E> {
    fn default() -> Self {
        Self::new()
    }
}
