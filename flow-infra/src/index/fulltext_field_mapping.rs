use std::collections::HashMap;

/// 全文搜索字段映射配置
/// 定义哪些索引字段应该使用全文搜索引擎
pub struct FulltextFieldMapping {
    /// 字段映射：index_name -> doc_type
    /// 如果字段在映射中，表示该字段应该使用全文搜索
    /// doc_type用于过滤搜索结果，例如："post.content.halo.run"
    mappings: HashMap<String, Vec<String>>,
}

impl FulltextFieldMapping {
    /// 创建默认的字段映射配置
    pub fn default() -> Self {
        let mut mappings = HashMap::new();
        
        // Post字段映射
        mappings.insert("spec.title".to_string(), vec!["post.content.halo.run".to_string()]);
        mappings.insert("status.excerpt".to_string(), vec!["post.content.halo.run".to_string()]);
        
        // SinglePage字段映射
        mappings.insert("spec.title".to_string(), vec!["singlepage.content.halo.run".to_string()]);
        
        // 注意：spec.content字段通常不在索引中，而是通过全文搜索直接搜索content字段
        
        Self { mappings }
    }
    
    /// 检查字段是否应该使用全文搜索
    pub fn is_fulltext_field(&self, index_name: &str) -> bool {
        self.mappings.contains_key(index_name)
    }
    
    /// 获取字段对应的文档类型列表
    pub fn get_doc_types(&self, index_name: &str) -> Option<&Vec<String>> {
        self.mappings.get(index_name)
    }
    
    /// 检查字段是否匹配指定的文档类型
    pub fn matches_doc_type(&self, index_name: &str, doc_type: &str) -> bool {
        if let Some(doc_types) = self.get_doc_types(index_name) {
            doc_types.contains(&doc_type.to_string())
        } else {
            false
        }
    }
}

impl Default for FulltextFieldMapping {
    fn default() -> Self {
        Self::default()
    }
}

