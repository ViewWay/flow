use flow_api::theme::Finder;
use async_trait::async_trait;
use serde_json::Value;

/// PostFinder - 在模板中查询Post数据
/// 注意：由于Finder需要在模板引擎中使用，而模板引擎通常是同步的，
/// 所以Finder的实现需要特殊处理（可能需要预加载数据或使用同步接口）
pub struct PostFinder {
    // TODO: 添加PostService引用（需要避免循环依赖）
    // 可以考虑使用ExtensionClient直接查询
}

impl PostFinder {
    pub fn new() -> Self {
        Self {}
    }
    
    /// 获取Post（异步方法，需要在模板引擎中特殊处理）
    pub async fn get_by_name(&self, _name: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现Post查询逻辑
        // 由于模板引擎是同步的，这里需要特殊处理
        // 可以考虑：
        // 1. 在模板渲染前预加载数据
        // 2. 使用同步的缓存查询
        // 3. 使用特殊的异步模板引擎
        Ok(serde_json::json!({}))
    }
}

impl Default for PostFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Finder for PostFinder {
    fn name(&self) -> &str {
        "postFinder"
    }
}

/// ThemeFinder - 在模板中查询Theme数据
pub struct ThemeFinder;

impl ThemeFinder {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Finder for ThemeFinder {
    fn name(&self) -> &str {
        "themeFinder"
    }
}

impl Default for ThemeFinder {
    fn default() -> Self {
        Self::new()
    }
}

