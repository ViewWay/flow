use flow_api::search::{HaloDocument, SearchOption, SearchResult, SearchEngine};
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::{BooleanQuery, Occur, QueryParser, TermQuery},
    schema::{Field, IndexRecordOption},
    Index, IndexReader, IndexWriter, ReloadPolicy, Searcher, Term,
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
        
        // 转换结果
        let mut hits = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let mut halo_doc = self.doc_converter.convert(&retrieved_doc);
            
            // 应用高亮（简化实现，实际应该使用Tantivy的高亮功能）
            // 这里只是简单替换关键词
            if !option.keyword.is_empty() {
                halo_doc.title = highlight_text(&halo_doc.title, &option.keyword, &option.highlight_pre_tag, &option.highlight_post_tag);
                if let Some(ref mut desc) = halo_doc.description {
                    *desc = highlight_text(desc, &option.keyword, &option.highlight_pre_tag, &option.highlight_post_tag);
                }
                halo_doc.content = highlight_text(&halo_doc.content, &option.keyword, &option.highlight_pre_tag, &option.highlight_post_tag);
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

/// 简单的文本高亮实现
fn highlight_text(text: &str, keyword: &str, pre_tag: &str, post_tag: &str) -> String {
    if keyword.is_empty() {
        return text.to_string();
    }
    
    // 简单的区分大小写替换（实际应该使用更智能的匹配）
    text.replace(keyword, &format!("{}{}{}", pre_tag, keyword, post_tag))
}

