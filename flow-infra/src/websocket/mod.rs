use flow_api::extension::GroupVersionKind;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// WebSocket端点定义（抽象层）
/// 具体的WebSocket处理在flow-web层实现
/// 
/// 注意：handler方法在flow-web层通过扩展trait实现，
/// 因为需要Axum的WebSocket类型，避免循环依赖
pub trait WebSocketEndpoint: Send + Sync {
    /// 获取URL路径（在group version之后的部分）
    fn url_path(&self) -> &str;
    
    /// 获取Group和Version
    fn group_version(&self) -> GroupVersionKind;
    
    /// 获取Any引用，用于类型擦除和downcast
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// 获取类型ID，用于路由到正确的handler
    /// 默认实现返回类型名称的字符串
    fn type_id(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// WebSocket端点管理器
/// 负责注册、注销和查找WebSocket端点
#[derive(Clone)]
pub struct WebSocketEndpointManager {
    /// 存储端点：路径 -> 端点
    /// 路径格式：{group}/{version}/{url_path}
    endpoints: Arc<RwLock<HashMap<String, Arc<dyn WebSocketEndpoint>>>>,
}

impl WebSocketEndpointManager {
    pub fn new() -> Self {
        Self {
            endpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册WebSocket端点
    pub async fn register(&self, endpoint: Arc<dyn WebSocketEndpoint>) {
        let path = self.build_path(&endpoint.group_version(), endpoint.url_path());
        let mut endpoints = self.endpoints.write().await;
        endpoints.insert(path, endpoint);
    }
    
    /// 批量注册端点
    pub async fn register_many(&self, endpoints: Vec<Arc<dyn WebSocketEndpoint>>) {
        let mut map = self.endpoints.write().await;
        for endpoint in endpoints {
            let path = self.build_path(&endpoint.group_version(), endpoint.url_path());
            map.insert(path, endpoint);
        }
    }
    
    /// 注销WebSocket端点
    pub async fn unregister(&self, group_version: &GroupVersionKind, url_path: &str) {
        let path = self.build_path(group_version, url_path);
        let mut endpoints = self.endpoints.write().await;
        endpoints.remove(&path);
    }
    
    /// 批量注销端点
    pub async fn unregister_many(&self, endpoints: Vec<(GroupVersionKind, String)>) {
        let mut map = self.endpoints.write().await;
        for (gv, url_path) in endpoints {
            let path = self.build_path(&gv, &url_path);
            map.remove(&path);
        }
    }
    
    /// 查找端点
    pub async fn find(&self, path: &str) -> Option<Arc<dyn WebSocketEndpoint>> {
        let endpoints = self.endpoints.read().await;
        endpoints.get(path).cloned()
    }
    
    /// 构建完整路径
    fn build_path(&self, group_version: &GroupVersionKind, url_path: &str) -> String {
        format!("{}/{}/{}", group_version.group, group_version.version, url_path.trim_start_matches('/'))
    }
    
    /// 获取所有已注册的端点路径
    pub async fn list_paths(&self) -> Vec<String> {
        let endpoints = self.endpoints.read().await;
        endpoints.keys().cloned().collect()
    }
}

impl Default for WebSocketEndpointManager {
    fn default() -> Self {
        Self::new()
    }
}

