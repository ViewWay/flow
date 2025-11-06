use flow_api::search::{HaloDocument, SearchOption, SearchResult, SearchEngine};
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::{BooleanQuery, Occur, QueryParser, TermQuery, Query},
    schema::{Field, IndexRecordOption, Value},
    Index, IndexReader, IndexWriter, ReloadPolicy, Searcher, Term,
    snippet::SnippetGenerator,
};
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;
use anyhow::{Context, Result};
use tracing::{debug, info};

use super::converter::{HaloDocumentConverter, DocumentConverter};

/// Tantivy搜索引擎实现
pub struct TantivySearchEngine {
    index: Arc<Index>,
    reader: Arc<RwLock<IndexReader>>,
    writer: Arc<AsyncRwLock<IndexWriter>>,
    converter: Arc<HaloDocumentConverter>,
    doc_converter: Arc<DocumentConverter>,
    id_field: Field,
    title_field: Field,
    description_field: Field,
    content_field: Field,
    doc_type_field: Field,
    owner_name_field: Field,
    category_field: Field,
    tag_field: Field,
    recycled_field: Field,
    exposed_field: Field,
    published_field: Field,
}

impl TantivySearchEngine {
    /// 创建新的Tantivy搜索引擎实例
    pub async fn new(index_path: impl AsRef<Path>) -> Result<Self> {
        let index_path = index_path.as_ref();
        
        // 确保索引目录存在
        std::fs::create_dir_all(index_path)
            .context("Failed to create index directory")?;
        
        let converter = Arc::new(HaloDocumentConverter::new());
        let schema = converter.schema().clone();
        
        // 打开或创建索引
        let directory = MmapDirectory::open(index_path)
            .context("Failed to open index directory")?;
        
        let index = Index::open_or_create(directory, schema.clone())
            .context("Failed to open or create index")?;
        
        let reader = Arc::new(RwLock::new(
            index.reader_builder()
                .reload_policy(ReloadPolicy::Manual)
                .try_into()
                .context("Failed to create index reader")?
        ));
        
        let writer = Arc::new(AsyncRwLock::new(
            index.writer(50_000_000)
                .context("Failed to create index writer")?
        ));
        
        let doc_converter = Arc::new(DocumentConverter::new(&converter));
        
        // 获取字段引用
        let id_field = converter.id_field;
        let title_field = converter.title_field;
        let description_field = converter.description_field;
        let content_field = converter.content_field;
        let doc_type_field = converter.doc_type_field;
        let owner_name_field = converter.owner_name_field;
        let category_field = converter.category_field;
        let tag_field = converter.tag_field;
        let recycled_field = converter.recycled_field;
        let exposed_field = converter.exposed_field;
        let published_field = converter.published_field;
        
        info!("Initialized Tantivy search engine at {:?}", index_path);
        
        Ok(Self {
            index: Arc::new(index),
            reader,
            writer,
            converter,
            doc_converter,
            id_field,
            title_field,
            description_field,
            content_field,
            doc_type_field,
            owner_name_field,
            category_field,
            tag_field,
            recycled_field,
            exposed_field,
            published_field,
        })
    }
    
    /// 刷新索引读取器
    async fn refresh_reader(&self) -> Result<()> {
        let reader_guard = self.reader.write().unwrap();
        reader_guard.reload()?;
        Ok(())
    }
    
    /// 获取搜索器
    fn get_searcher(&self) -> Result<Searcher> {
        let reader_guard = self.reader.read().unwrap();
        Ok(reader_guard.searcher())
    }
}

#[async_trait::async_trait]
impl SearchEngine for TantivySearchEngine {
    fn available(&self) -> bool {
        true
    }
    
    async fn add_or_update(&self, documents: Vec<HaloDocument>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if documents.is_empty() {
            return Ok(());
        }
        
        let mut writer = self.writer.write().await;
        
        // 构建删除查询（基于文档ID）
        let delete_terms: Vec<Term> = documents.iter()
            .map(|doc| Term::from_field_text(self.id_field, &doc.id))
            .collect();
        
        // 删除旧文档
        for term in &delete_terms {
            writer.delete_term(term.clone());
        }
        
        // 添加新文档
        for halo_doc in &documents {
            let tantivy_doc = self.converter.convert(halo_doc);
            writer.add_document(tantivy_doc)?;
        }
        
        // 提交更改
        writer.commit()?;
        drop(writer);
        
        // 刷新读取器
        self.refresh_reader().await?;
        
        debug!("Added or updated {} documents in search index", documents.len());
        Ok(())
    }
    
    async fn delete_document(&self, doc_ids: Vec<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if doc_ids.is_empty() {
            return Ok(());
        }
        
        let mut writer = self.writer.write().await;
        
        for doc_id in doc_ids {
            let term = Term::from_field_text(self.id_field, &doc_id);
            writer.delete_term(term);
        }
        
        writer.commit()?;
        drop(writer);
        
        self.refresh_reader().await?;
        
        debug!("Deleted documents from search index");
        Ok(())
    }
    
    async fn delete_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut writer = self.writer.write().await;
        writer.delete_all_documents()?;
        writer.commit()?;
        drop(writer);
        
        self.refresh_reader().await?;
        
        info!("Deleted all documents from search index");
        Ok(())
    }
    
    async fn search(&self, option: SearchOption) -> Result<SearchResult, Box<dyn std::error::Error + Send + Sync>> {
        let searcher = self.get_searcher()?;
        
        // 检查索引是否为空
        if searcher.num_docs() == 0 {
            return Ok(SearchResult {
                hits: vec![],
                keyword: option.keyword.clone(),
                total: 0,
                limit: option.limit,
                processing_time_millis: 0,
            });
        }
        
        // 创建查询解析器（支持多字段搜索）
        let mut query_parser = QueryParser::for_index(
            &self.index,
            vec![self.title_field, self.description_field, self.content_field]
        );
        
        // 设置字段权重
        query_parser.set_field_boost(self.title_field, 1.0);
        query_parser.set_field_boost(self.description_field, 0.5);
        query_parser.set_field_boost(self.content_field, 0.2);
        
        // 解析关键词查询
        let keyword_query = query_parser.parse_query(&option.keyword)
            .map_err(|e| format!("Failed to parse query: {}", e))?;
        
        // 构建布尔查询
        let mut query_clauses = vec![(Occur::Must, keyword_query)];
        
        // 添加过滤条件
        if let Some(filter_exposed) = option.filter_exposed {
            let term = Term::from_field_text(self.exposed_field, &filter_exposed.to_string());
            query_clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
        }
        
        if let Some(filter_recycled) = option.filter_recycled {
            let term = Term::from_field_text(self.recycled_field, &filter_recycled.to_string());
            query_clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
        }
        
        if let Some(filter_published) = option.filter_published {
            let term = Term::from_field_text(self.published_field, &filter_published.to_string());
            query_clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
        }
        
        if let Some(include_types) = &option.include_types {
            if !include_types.is_empty() {
                let mut type_clauses: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
                for doc_type in include_types {
                    let term = Term::from_field_text(self.doc_type_field, doc_type);
                    type_clauses.push((Occur::Should, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
                }
                if type_clauses.len() == 1 {
                    query_clauses.push(type_clauses.remove(0));
                } else if type_clauses.len() > 1 {
                    query_clauses.push((Occur::Must, Box::new(BooleanQuery::new(type_clauses))));
                }
            }
        }
        
        if let Some(include_owner_names) = &option.include_owner_names {
            if !include_owner_names.is_empty() {
                let mut owner_clauses: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
                for owner_name in include_owner_names {
                    let term = Term::from_field_text(self.owner_name_field, owner_name);
                    owner_clauses.push((Occur::Should, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
                }
                if owner_clauses.len() == 1 {
                    query_clauses.push(owner_clauses.remove(0));
                } else if owner_clauses.len() > 1 {
                    query_clauses.push((Occur::Must, Box::new(BooleanQuery::new(owner_clauses))));
                }
            }
        }
        
        if let Some(include_tag_names) = &option.include_tag_names {
            if !include_tag_names.is_empty() {
                for tag_name in include_tag_names {
                    let term = Term::from_field_text(self.tag_field, tag_name);
                    query_clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
                }
            }
        }
        
        if let Some(include_category_names) = &option.include_category_names {
            if !include_category_names.is_empty() {
                for category_name in include_category_names {
                    let term = Term::from_field_text(self.category_field, category_name);
                    query_clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
                }
            }
        }
        
        let final_query = if query_clauses.len() == 1 {
            query_clauses.remove(0).1
        } else {
            Box::new(BooleanQuery::new(query_clauses))
        };
        
        // 执行搜索
        let start_time = std::time::Instant::now();
        let top_docs = searcher.search(&*final_query, &TopDocs::with_limit(option.limit as usize))?;
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // 创建高亮生成器（如果有关键词）
        let use_highlighting = !option.keyword.is_empty();
        
        // 为每个字段创建高亮生成器
        let title_snippet_gen = if use_highlighting {
            SnippetGenerator::create(&searcher, &*final_query, self.title_field).ok()
        } else {
            None
        };
        
        let desc_snippet_gen = if use_highlighting {
            SnippetGenerator::create(&searcher, &*final_query, self.description_field).ok()
        } else {
            None
        };
        
        let content_snippet_gen = if use_highlighting {
            SnippetGenerator::create(&searcher, &*final_query, self.content_field).ok()
        } else {
            None
        };
        
        // 转换结果并应用高亮
        let mut hits = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let mut halo_doc = self.doc_converter.convert(&retrieved_doc);
            
            // 使用 Tantivy 原生高亮功能
            if use_highlighting {
                // 高亮 title 字段
                if let Some(ref snippet_gen) = title_snippet_gen {
                    if let Some(title_value) = retrieved_doc.get_first(self.title_field)
                        .and_then(|v| v.as_str())
                    {
                        match highlight_field(
                            snippet_gen,
                            title_value,
                            &option.highlight_pre_tag,
                            &option.highlight_post_tag,
                        ) {
                            Ok(highlighted) => halo_doc.title = highlighted,
                            Err(_) => {
                                debug!("Failed to highlight title field, using original text");
                            }
                        }
                    }
                }
                
                // 高亮 description 字段
                if let Some(ref snippet_gen) = desc_snippet_gen {
                    if let Some(desc_value) = retrieved_doc.get_first(self.description_field)
                        .and_then(|v| v.as_str())
                    {
                        match highlight_field(
                            snippet_gen,
                            desc_value,
                            &option.highlight_pre_tag,
                            &option.highlight_post_tag,
                        ) {
                            Ok(highlighted) => halo_doc.description = Some(highlighted),
                            Err(_) => {
                                debug!("Failed to highlight description field, using original text");
                            }
                        }
                    }
                }
                
                // 高亮 content 字段
                if let Some(ref snippet_gen) = content_snippet_gen {
                    if let Some(content_value) = retrieved_doc.get_first(self.content_field)
                        .and_then(|v| v.as_str())
                    {
                        match highlight_field(
                            snippet_gen,
                            content_value,
                            &option.highlight_pre_tag,
                            &option.highlight_post_tag,
                        ) {
                            Ok(highlighted) => halo_doc.content = highlighted,
                            Err(_) => {
                                debug!("Failed to highlight content field, using original text");
                            }
                        }
                    }
                }
            }
            
            hits.push(halo_doc);
        }
        
        Ok(SearchResult {
            hits,
            keyword: option.keyword,
            total: searcher.num_docs(),
            limit: option.limit,
            processing_time_millis: processing_time,
        })
    }
}

/// 使用 Tantivy 原生高亮功能高亮字段
fn highlight_field(
    snippet_gen: &SnippetGenerator,
    text: &str,
    pre_tag: &str,
    post_tag: &str,
) -> Result<String> {
    // 生成片段
    let mut snippet = snippet_gen.snippet(text);
    
    // 设置自定义的高亮标签
    snippet.set_snippet_prefix_postfix(pre_tag, post_tag);
    
    // 手动构建高亮文本（不使用 to_html，因为我们需要原始文本而不是转义的 HTML）
    let fragment = snippet.fragment();
    let highlighted_ranges = snippet.highlighted();
    
    if highlighted_ranges.is_empty() {
        // 如果没有匹配，返回原始文本
        return Ok(text.to_string());
    }
    
    let mut result = String::new();
    let mut last_end = 0;
    
    // 按顺序处理所有高亮范围
    let mut sorted_ranges: Vec<_> = highlighted_ranges.iter().collect();
    sorted_ranges.sort_by_key(|r| r.start);
    
    for range in sorted_ranges {
        // 添加高亮前的文本
        if range.start > last_end {
            result.push_str(&fragment[last_end..range.start]);
        }
        // 添加高亮标签和匹配文本
        result.push_str(pre_tag);
        result.push_str(&fragment[range.clone()]);
        result.push_str(post_tag);
        last_end = range.end;
    }
    
    // 添加剩余的文本
    if last_end < fragment.len() {
        result.push_str(&fragment[last_end..]);
    }
    
    Ok(result)
}

