use flow_api::extension::Extension;
use flow_api::extension::query::Condition;
use flow_api::extension::index::LabelIndexQuery;
use flow_api::search::{SearchEngine, SearchOption};
use crate::index::indices::Indices;
use crate::index::fulltext_field_mapping::FulltextFieldMapping;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::warn;

/// QueryVisitor 遍历查询条件并执行查询
pub struct QueryVisitor<E: Extension + 'static> {
    result: HashSet<String>,
    indices: Arc<Indices<E>>,
    search_engine: Option<Arc<dyn SearchEngine>>,
    fulltext_mapping: Arc<FulltextFieldMapping>,
    doc_type: Option<String>, // 可选的文档类型，如果提供则启用全文搜索
}

impl<E: Extension + 'static> QueryVisitor<E> {
    pub fn new(
        indices: Arc<Indices<E>>,
        search_engine: Option<Arc<dyn SearchEngine>>,
        fulltext_mapping: Arc<FulltextFieldMapping>,
    ) -> Self {
        Self {
            result: HashSet::new(),
            indices,
            search_engine,
            fulltext_mapping,
            doc_type: None,
        }
    }
    
    /// 创建带有文档类型的 QueryVisitor（用于支持全文搜索）
    pub fn with_doc_type(
        indices: Arc<Indices<E>>,
        search_engine: Option<Arc<dyn SearchEngine>>,
        fulltext_mapping: Arc<FulltextFieldMapping>,
        doc_type: String,
    ) -> Self {
        Self {
            result: HashSet::new(),
            indices,
            search_engine,
            fulltext_mapping,
            doc_type: Some(doc_type),
        }
    }
    
    pub async fn visit(&mut self, condition: &Condition) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match condition {
            Condition::Empty => {
                // 空条件匹配所有
                // 使用标签索引获取所有主键
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                self.result.extend(all_keys);
            }
            
            Condition::And { left, right } => {
                Box::pin(self.visit(left)).await?;
                if !self.result.is_empty() {
                    let mut right_result = HashSet::new();
                    Self::visit_condition(
                        right,
                        &self.indices,
                        self.search_engine.as_ref(),
                        &self.fulltext_mapping,
                        self.doc_type.as_ref(),
                        &mut right_result,
                    ).await?;
                    self.result = self.result.intersection(&right_result).cloned().collect();
                }
            }
            
            Condition::Or { left, right } => {
                Box::pin(self.visit(left)).await?;
                let mut right_result = HashSet::new();
                Self::visit_condition(
                    right,
                    &self.indices,
                    self.search_engine.as_ref(),
                    &self.fulltext_mapping,
                    self.doc_type.as_ref(),
                    &mut right_result,
                ).await?;
                self.result.extend(right_result);
            }
            
            Condition::Not { condition } => {
                // 获取所有主键，然后移除匹配的
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                
                let mut matched = HashSet::new();
                Self::visit_condition(
                    condition,
                    &self.indices,
                    self.search_engine.as_ref(),
                    &self.fulltext_mapping,
                    self.doc_type.as_ref(),
                    &mut matched,
                ).await?;
                
                self.result = all_keys.difference(&matched).cloned().collect();
            }
            
            Condition::Equal { index_name, value } => {
                if index_name == "metadata.labels" {
                    // 这是标签查询，不应该在这里
                    return Err("Label queries should use LabelEquals condition".into());
                }
                let result = self.indices.query_equal(index_name, value)?;
                self.result.extend(result);
            }
            
            Condition::NotEqual { index_name, value } => {
                if index_name == "metadata.labels" {
                    return Err("Label queries should use LabelNotEquals condition".into());
                }
                let result = self.indices.query_not_equal(index_name, value)?;
                self.result.extend(result);
            }
            
            Condition::In { index_name, values } => {
                if index_name == "metadata.labels" {
                    return Err("Label queries should use LabelIn condition".into());
                }
                let result = self.indices.query_in(index_name, values)?;
                self.result.extend(result);
            }
            
            Condition::NotIn { index_name, values } => {
                if index_name == "metadata.labels" {
                    return Err("Label queries should use LabelNotIn condition".into());
                }
                let result = self.indices.query_not_in(index_name, values)?;
                self.result.extend(result);
            }
            
            Condition::LessThan { index_name, bound, inclusive } => {
                let result = self.indices.query_less_than(index_name, bound, *inclusive)?;
                self.result.extend(result);
            }
            
            Condition::GreaterThan { index_name, bound, inclusive } => {
                let result = self.indices.query_greater_than(index_name, bound, *inclusive)?;
                self.result.extend(result);
            }
            
            Condition::Between { index_name, from_key, from_inclusive, to_key, to_inclusive } => {
                let result = self.indices.query_between(
                    index_name,
                    from_key,
                    *from_inclusive,
                    to_key,
                    *to_inclusive,
                )?;
                self.result.extend(result);
            }
            
            Condition::NotBetween { index_name, from_key, from_inclusive, to_key, to_inclusive } => {
                let result = self.indices.query_not_between(
                    index_name,
                    from_key,
                    *from_inclusive,
                    to_key,
                    *to_inclusive,
                )?;
                self.result.extend(result);
            }
            
            Condition::IsNull { index_name } => {
                // IS NULL查询：获取所有主键，然后移除索引中存在的
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                let indexed_keys = self.indices.query_all(index_name)
                    .unwrap_or_default();
                self.result = all_keys.difference(&indexed_keys).cloned().collect();
            }
            
            Condition::IsNotNull { index_name } => {
                // IS NOT NULL查询：直接获取索引中的所有主键
                let result = self.indices.query_all(index_name)?;
                self.result.extend(result);
            }
            
            Condition::Contains { index_name, value } => {
                // Contains查询：检查是否是全文搜索字段
                if self.fulltext_mapping.is_fulltext_field(index_name) {
                    // 使用 SearchEngine 进行全文搜索
                    if let Some(ref engine) = self.search_engine {
                        if engine.available() {
                            // 检查是否有文档类型（如果提供了，说明类型支持全文搜索）
                            if let Some(ref doc_type) = self.doc_type {
                                // 构建 SearchOption
                                let search_option = SearchOption {
                                    keyword: value.clone(),
                                    limit: 10000, // 足够大的限制，确保获取所有匹配结果
                                    highlight_pre_tag: "<B>".to_string(),
                                    highlight_post_tag: "</B>".to_string(),
                                    filter_exposed: None,
                                    filter_recycled: None,
                                    filter_published: None,
                                    include_types: Some(vec![doc_type.clone()]),
                                    include_owner_names: None,
                                    include_category_names: None,
                                    include_tag_names: None,
                                    sort_by: None,
                                    sort_order: flow_api::search::SortOrder::Desc,
                                    annotations: None,
                                };
                                
                                // 执行搜索
                                match engine.search(search_option).await {
                                    Ok(search_result) => {
                                        // 提取主键集合（metadata_name）
                                        let matched_keys: HashSet<String> = search_result.hits
                                            .iter()
                                            .map(|doc| doc.metadata_name.clone())
                                            .collect();
                                        self.result.extend(matched_keys);
                                    }
                                    Err(e) => {
                                        warn!("SearchEngine search failed for field {}: {}, falling back to string matching", index_name, e);
                                        // 搜索失败，回退到字符串匹配
                                        match self.indices.query_string_contains(index_name, &value) {
                                            Ok(matched_keys) => {
                                                self.result.extend(matched_keys);
                                            }
                                            Err(_) => {
                                                let all_keys = self.indices.query_all(index_name).unwrap_or_default();
                                                self.result.extend(all_keys);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // 没有提供文档类型，回退到字符串匹配
                                warn!("No doc_type provided for QueryVisitor, falling back to string matching for field {}", index_name);
                                match self.indices.query_string_contains(index_name, &value) {
                                    Ok(matched_keys) => {
                                        self.result.extend(matched_keys);
                                    }
                                    Err(_) => {
                                        let all_keys = self.indices.query_all(index_name).unwrap_or_default();
                                        self.result.extend(all_keys);
                                    }
                                }
                            }
                        } else {
                            // SearchEngine 不可用，回退到字符串匹配
                            warn!("SearchEngine not available, falling back to string matching for field {}", index_name);
                            match self.indices.query_string_contains(index_name, &value) {
                                Ok(matched_keys) => {
                                    self.result.extend(matched_keys);
                                }
                                Err(_) => {
                                    let all_keys = self.indices.query_all(index_name).unwrap_or_default();
                                    self.result.extend(all_keys);
                                }
                            }
                        }
                    } else {
                        // 没有 SearchEngine，回退到字符串匹配
                        match self.indices.query_string_contains(index_name, &value) {
                            Ok(matched_keys) => {
                                self.result.extend(matched_keys);
                            }
                            Err(_) => {
                                let all_keys = self.indices.query_all(index_name).unwrap_or_default();
                                self.result.extend(all_keys);
                            }
                        }
                    }
                } else {
                    // 不是全文搜索字段，使用字符串匹配
                    match self.indices.query_string_contains(index_name, &value) {
                        Ok(matched_keys) => {
                            self.result.extend(matched_keys);
                        }
                        Err(_) => {
                            // 如果不是字符串类型索引，回退到获取所有键
                            let all_keys = self.indices.query_all(index_name).unwrap_or_default();
                            self.result.extend(all_keys);
                        }
                    }
                }
            }
            
            Condition::LabelExists { label_key } => {
                let label_index = self.indices.label_index();
                let result = label_index.exists(label_key);
                self.result.extend(result);
            }
            
            Condition::LabelNotExists { label_key } => {
                // 获取所有主键，然后移除有该标签的
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                let exists_result = label_index.exists(label_key);
                self.result = all_keys.difference(&exists_result).cloned().collect();
            }
            
            Condition::LabelEquals { label_key, label_value } => {
                let label_index = self.indices.label_index();
                let result = label_index.equal(label_key, label_value);
                self.result.extend(result);
            }
            
            Condition::LabelNotEquals { label_key, label_value } => {
                let label_index = self.indices.label_index();
                let result = label_index.not_equal(label_key, label_value);
                self.result.extend(result);
            }
            
            Condition::LabelIn { label_key, label_values } => {
                let label_index = self.indices.label_index();
                let result = label_index.in_set(label_key, label_values);
                self.result.extend(result);
            }
            
            Condition::LabelNotIn { label_key, label_values } => {
                let label_index = self.indices.label_index();
                let result = label_index.not_in_set(label_key, label_values);
                self.result.extend(result);
            }
        }
        
        Ok(())
    }
    
    async fn visit_condition(
        condition: &Condition,
        indices: &Arc<Indices<E>>,
        search_engine: Option<&Arc<dyn SearchEngine>>,
        fulltext_mapping: &Arc<FulltextFieldMapping>,
        doc_type: Option<&String>,
        result: &mut HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 创建 QueryVisitor，如果提供了 doc_type 则使用 with_doc_type
        let mut visitor = if let Some(doc_type) = doc_type {
            Self::with_doc_type(
                Arc::clone(indices),
                search_engine.map(|e| Arc::clone(e)),
                Arc::clone(fulltext_mapping),
                doc_type.clone(),
            )
        } else {
            Self::new(
                Arc::clone(indices),
                search_engine.map(|e| Arc::clone(e)),
                Arc::clone(fulltext_mapping),
            )
        };
        Box::pin(visitor.visit(condition)).await?;
        result.extend(visitor.result);
        Ok(())
    }
    
    pub fn get_result(self) -> HashSet<String> {
        self.result
    }
}

#[cfg(test)]
mod tests {
    use flow_api::search::{SearchEngine, SearchOption, SearchResult, HaloDocument};
    use async_trait::async_trait;
    use chrono::Utc;

    /// MockSearchEngine 用于测试
    pub struct MockSearchEngine {
        available: bool,
        search_results: Vec<HaloDocument>,
    }

    impl MockSearchEngine {
        pub fn new(available: bool) -> Self {
            Self {
                available,
                search_results: Vec::new(),
            }
        }

        pub fn with_results(mut self, results: Vec<HaloDocument>) -> Self {
            self.search_results = results;
            self
        }
    }

    #[async_trait]
    impl SearchEngine for MockSearchEngine {
        fn available(&self) -> bool {
            self.available
        }

        async fn add_or_update(&self, _documents: Vec<HaloDocument>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }

        async fn delete_document(&self, _doc_ids: Vec<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }

        async fn delete_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }

        async fn search(&self, option: SearchOption) -> Result<SearchResult, Box<dyn std::error::Error + Send + Sync>> {
            // 根据 option 过滤结果
            let mut filtered_results = self.search_results.clone();
            
            // 如果指定了 include_types，只返回匹配的文档
            if let Some(ref include_types) = option.include_types {
                filtered_results.retain(|doc| include_types.contains(&doc.doc_type));
            }
            
            let total = filtered_results.len() as u64;
            Ok(SearchResult {
                hits: filtered_results,
                keyword: option.keyword,
                total,
                limit: option.limit,
                processing_time_millis: 0,
            })
        }
    }

    /// 测试 MockSearchEngine 的基本功能
    #[tokio::test]
    async fn test_mock_search_engine() {
        let doc1 = HaloDocument {
            id: "post.content.halo.run-test-1".to_string(),
            metadata_name: "test-1".to_string(),
            annotations: None,
            title: "Test Post 1".to_string(),
            description: Some("Test description".to_string()),
            content: "Test content".to_string(),
            categories: None,
            tags: None,
            published: true,
            recycled: false,
            exposed: true,
            owner_name: "admin".to_string(),
            creation_timestamp: Some(Utc::now()),
            update_timestamp: Some(Utc::now()),
            permalink: "/test-1".to_string(),
            doc_type: "post.content.halo.run".to_string(),
        };
        
        let engine = MockSearchEngine::new(true)
            .with_results(vec![doc1.clone()]);
        
        assert!(engine.available());
        
        let option = SearchOption {
            keyword: "test".to_string(),
            limit: 10,
            highlight_pre_tag: "<B>".to_string(),
            highlight_post_tag: "</B>".to_string(),
            filter_exposed: None,
            filter_recycled: None,
            filter_published: None,
            include_types: Some(vec!["post.content.halo.run".to_string()]),
            include_owner_names: None,
            include_category_names: None,
            include_tag_names: None,
            sort_by: None,
            sort_order: flow_api::search::SortOrder::Desc,
            annotations: None,
        };
        
        let result = engine.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].metadata_name, "test-1");
    }
}

