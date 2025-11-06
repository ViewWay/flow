use flow_api::theme::{Finder, FinderRegistry};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 默认Finder注册表实现
/// 使用Arc存储Finder，以便可以安全地返回和克隆
#[derive(Clone)]
pub struct DefaultFinderRegistry {
    finders: Arc<RwLock<HashMap<String, Arc<dyn Finder>>>>,
}

impl DefaultFinderRegistry {
    pub fn new() -> Self {
        Self {
            finders: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl FinderRegistry for DefaultFinderRegistry {
    fn register(&self, name: String, finder: Box<dyn Finder>) {
        let mut finders = self.finders.write().unwrap();
        // 将Box转换为Arc以便可以克隆
        finders.insert(name, Arc::from(finder));
    }
    
    fn get(&self, name: &str) -> Option<Arc<dyn Finder>> {
        let finders = self.finders.read().unwrap();
        // 克隆Arc以返回，这样不会移动数据
        finders.get(name).map(|finder| Arc::clone(finder))
    }
    
    fn get_all(&self) -> HashMap<String, Arc<dyn Finder>> {
        let finders = self.finders.read().unwrap();
        // 克隆所有Arc，构建新的HashMap
        finders.iter()
            .map(|(name, finder)| (name.clone(), Arc::clone(finder)))
            .collect()
    }
    
    fn remove(&self, name: &str) {
        let mut finders = self.finders.write().unwrap();
        finders.remove(name);
    }
}

impl Default for DefaultFinderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

