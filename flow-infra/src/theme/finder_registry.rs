use flow_api::theme::{Finder, FinderRegistry};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 默认Finder注册表实现
#[derive(Clone)]
pub struct DefaultFinderRegistry {
    finders: Arc<RwLock<HashMap<String, Box<dyn Finder>>>>,
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
        finders.insert(name, finder);
    }
    
    fn get(&self, name: &str) -> Option<&dyn Finder> {
        let finders = self.finders.read().unwrap();
        // 注意：这里返回的是引用，但由于RwLock的限制，我们需要返回Option
        // 实际使用中，应该通过get_all获取
        None // TODO: 需要重新设计以支持引用返回
    }
    
    fn get_all(&self) -> HashMap<String, Box<dyn Finder>> {
        let finders = self.finders.read().unwrap();
        // 由于无法克隆trait object，这里返回空map
        // 实际使用中需要通过其他方式访问
        HashMap::new()
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

