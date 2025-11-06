use flow_domain::theme::ThemeContext;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use moka::future::Cache;
use tera::{Tera, Context};

/// 模板引擎管理器
/// 管理每个主题的模板引擎实例（使用LRU缓存）
pub struct TemplateEngineManager {
    /// 模板引擎缓存（LRU，最多5个）
    engine_cache: Cache<String, Arc<dyn TemplateRenderer>>,
    /// 主题根目录
    theme_root: PathBuf,
}

/// 模板渲染器trait
pub trait TemplateRenderer: Send + Sync {
    /// 渲染模板
    fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// 模板上下文
#[derive(Debug, Clone)]
pub struct TemplateContext {
    /// 模型数据
    pub model: HashMap<String, serde_json::Value>,
    /// Finder数据（在模板中可调用）
    pub finders: HashMap<String, serde_json::Value>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self {
            model: HashMap::new(),
            finders: HashMap::new(),
        }
    }
    
    pub fn with_model(mut self, model: HashMap<String, serde_json::Value>) -> Self {
        self.model = model;
        self
    }
    
    pub fn with_finders(mut self, finders: HashMap<String, serde_json::Value>) -> Self {
        self.finders = finders;
        self
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngineManager {
    pub fn new(theme_root: PathBuf) -> Self {
        // 创建LRU缓存，最多5个模板引擎
        let cache = Cache::builder()
            .max_capacity(5)
            .build();
        
        Self {
            engine_cache: cache,
            theme_root,
        }
    }
    
    /// 获取模板引擎（从缓存或创建新的）
    pub async fn get_template_engine(&self, theme_context: &ThemeContext) -> Arc<dyn TemplateRenderer> {
        let cache_key = format!("{}:{}", theme_context.name, theme_context.active);
        
        let theme_ctx = theme_context.clone();
        self.engine_cache.get_with(cache_key.clone(), async move {
            // 创建新的模板引擎
            match TeraTemplateEngine::new(theme_ctx) {
                Ok(engine) => Arc::new(engine) as Arc<dyn TemplateRenderer>,
                Err(e) => {
                    // 如果创建失败，返回一个错误渲染器
                    Arc::new(ErrorTemplateEngine::new(e.to_string())) as Arc<dyn TemplateRenderer>
                }
            }
        }).await
    }
    
    /// 清除指定主题的缓存
    pub async fn clear_cache(&self, theme_name: &str) {
        // 清除所有相关的缓存项
        self.engine_cache.invalidate(&format!("{}:true", theme_name)).await;
        self.engine_cache.invalidate(&format!("{}:false", theme_name)).await;
    }
}

/// Tera模板引擎实现
/// 使用Tera作为运行时模板引擎，支持动态加载主题模板
struct TeraTemplateEngine {
    theme_context: ThemeContext,
    template_path: PathBuf,
    tera: Tera,
}

impl TeraTemplateEngine {
    fn new(theme_context: ThemeContext) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let template_path = theme_context.path.join("templates");
        
        // 创建Tera实例，从模板目录加载模板
        let mut tera = Tera::new(
            template_path.join("**/*.html")
                .to_str()
                .ok_or("Invalid template path")?
        )?;
        
        // 配置Tera
        tera.autoescape_on(vec![".html", ".htm", ".xml"]);
        
        Ok(Self {
            theme_context,
            template_path,
            tera,
        })
    }
}

impl TemplateRenderer for TeraTemplateEngine {
    fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 构建Tera上下文
        let mut tera_context = Context::new();
        
        // 添加模型数据
        for (key, value) in &context.model {
            tera_context.insert(key, value);
        }
        
        // 添加Finder数据
        for (key, value) in &context.finders {
            tera_context.insert(key, value);
        }
        
        // 添加主题信息
        tera_context.insert("theme", &self.theme_context.name);
        
        // 渲染模板
        let rendered = self.tera.render(template_name, &tera_context)?;
        Ok(rendered)
    }
}

/// 错误模板引擎（当无法创建Tera引擎时使用）
struct ErrorTemplateEngine {
    error: String,
}

impl ErrorTemplateEngine {
    fn new(error: String) -> Self {
        Self { error }
    }
}

impl TemplateRenderer for ErrorTemplateEngine {
    fn render(&self, template_name: &str, _context: &TemplateContext) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!("Template engine error: {} (Template: {})", self.error, template_name).into())
    }
}

