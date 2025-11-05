use flow_api::extension::Extension;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::indices::Indices;
use super::single_value_index::{SingleValueIndex, SingleValueIndexSpec};
use super::multi_value_index::{MultiValueIndex, MultiValueIndexSpec};

/// IndicesManager 管理所有扩展类型的索引
/// 
/// 使用类型安全的API，通过回调函数避免类型擦除问题
pub struct IndicesManager {
    /// 类型ID到Indices存储的映射
    indices_map: Arc<RwLock<HashMap<TypeId, IndicesStorage>>>,
}

/// IndicesStorage 存储Indices的Arc，提供类型安全的访问
enum IndicesStorage {
    /// 存储Indices<E>的Arc（通过类型擦除）
    /// 注意：这需要运行时类型检查来确保类型安全
    Any(Arc<dyn IndicesAny + Send + Sync>),
}

/// IndicesAny trait 用于类型擦除
trait IndicesAny: Send + Sync {
    fn type_id(&self) -> TypeId;
}

impl<E: Extension + 'static> IndicesAny for Indices<E> {
    fn type_id(&self) -> TypeId {
        TypeId::of::<E>()
    }
}

impl IndicesManager {
    pub fn new() -> Self {
        Self {
            indices_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 为指定类型添加索引
    pub fn add<E: Extension + 'static>(
        &self,
        single_specs: Vec<Box<dyn SingleValueIndexSpec<E, String> + Send + Sync>>,
        multi_specs: Vec<Box<dyn MultiValueIndexSpec<E, String> + Send + Sync>>,
    ) {
        let type_id = TypeId::of::<E>();
        let indices = Arc::new(Indices::<E>::new());
        
        // 添加单值索引（字符串类型）
        for spec in single_specs {
            let index: SingleValueIndex<E, String> = SingleValueIndex::new(spec);
            indices.add_string_index(index);
        }
        
        // 添加多值索引（字符串类型）
        for spec in multi_specs {
            let index: MultiValueIndex<E, String> = MultiValueIndex::new(spec);
            indices.add_string_multi_index(index);
        }
        
        self.indices_map
            .write()
            .unwrap()
            .insert(type_id, IndicesStorage::Any(indices));
    }
    
    /// 获取指定类型的Indices（使用回调函数避免类型擦除问题）
    pub fn with_indices<E: Extension + 'static, F, R>(
        &self,
        f: F,
    ) -> Result<R, String>
    where
        F: FnOnce(&Arc<Indices<E>>) -> R,
    {
        let type_id = TypeId::of::<E>();
        let map = self.indices_map.read().unwrap();
        
        let storage = map.get(&type_id)
            .ok_or_else(|| format!("No indices found for type"))?;
        
        match storage {
            IndicesStorage::Any(any_indices) => {
                // 验证类型ID匹配
                if any_indices.type_id() != type_id {
                    return Err("Type mismatch".to_string());
                }
                
                // 将Arc<dyn IndicesAny>转换为Arc<Indices<E>>
                // 这是安全的，因为：
                // 1. TypeId验证保证了类型匹配
                // 2. Arc的内部结构允许我们提取数据指针
                // 3. 我们不会减少原始Arc的引用计数
                let any_ptr = Arc::as_ptr(any_indices);
                let indices_ptr = any_ptr as *const Indices<E>;
                
                // 创建一个临时的Arc（增加引用计数）
                // 这样可以在闭包中使用，同时保持原始Arc有效
                unsafe {
                    // 增加引用计数，这样我们创建的Arc不会影响原始Arc
                    Arc::increment_strong_count(indices_ptr);
                    let indices_arc = Arc::from_raw(indices_ptr);
                    let result = f(&indices_arc);
                    // 释放我们创建的Arc（减少引用计数）
                    drop(indices_arc);
                    Ok(result)
                }
            }
        }
    }
    
    /// 获取指定类型的Indices的Arc（类型安全版本）
    pub fn get<E: Extension + 'static>(&self) -> Result<Arc<Indices<E>>, String> {
        self.with_indices(|indices| Arc::clone(indices))
    }
    
    /// 检查是否有指定类型的Indices
    pub fn has<E: Extension + 'static>(&self) -> bool {
        let type_id = TypeId::of::<E>();
        self.indices_map.read().unwrap().contains_key(&type_id)
    }
    
    /// 移除指定类型的Indices
    pub fn remove<E: Extension + 'static>(&self) {
        let type_id = TypeId::of::<E>();
        self.indices_map.write().unwrap().remove(&type_id);
    }
}

impl Default for IndicesManager {
    fn default() -> Self {
        Self::new()
    }
}
