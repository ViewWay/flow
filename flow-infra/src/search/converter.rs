use flow_api::search::HaloDocument;
use tantivy::schema::{Schema, Field, Value, STORED, TEXT};
use tantivy::TantivyDocument;
use chrono::DateTime;
use std::collections::HashMap;

/// 将HaloDocument转换为Tantivy Document的转换器
pub struct HaloDocumentConverter {
    schema: Schema,
    pub id_field: Field,
    pub metadata_name_field: Field,
    pub doc_type_field: Field,
    pub owner_name_field: Field,
    pub category_field: Field,
    pub tag_field: Field,
    pub title_field: Field,
    pub description_field: Field,
    pub content_field: Field,
    pub recycled_field: Field,
    pub exposed_field: Field,
    pub published_field: Field,
    pub annotations_field: Field,
    pub creation_timestamp_field: Field,
    pub update_timestamp_field: Field,
    pub permalink_field: Field,
}

impl HaloDocumentConverter {
    pub fn new() -> Self {
        let mut schema_builder = tantivy::schema::SchemaBuilder::new();
        
        // 字符串字段（不分词，用于过滤）
        let id_field = schema_builder.add_text_field("id", TEXT | STORED);
        let metadata_name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let doc_type_field = schema_builder.add_text_field("type", TEXT | STORED);
        let owner_name_field = schema_builder.add_text_field("ownerName", TEXT | STORED);
        let category_field = schema_builder.add_text_field("category", TEXT | STORED);
        let tag_field = schema_builder.add_text_field("tag", TEXT | STORED);
        let recycled_field = schema_builder.add_text_field("recycled", TEXT | STORED);
        let exposed_field = schema_builder.add_text_field("exposed", TEXT | STORED);
        let published_field = schema_builder.add_text_field("published", TEXT | STORED);
        let permalink_field = schema_builder.add_text_field("permalink", TEXT | STORED);
        
        // 文本字段（分词，用于搜索）
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        
        // 文本字段（用于存储annotations的JSON字符串）
        let annotations_field = schema_builder.add_text_field("annotations", TEXT | STORED);
        
        // 时间戳字段
        let creation_timestamp_field = schema_builder.add_i64_field("creationTimestamp", STORED);
        let update_timestamp_field = schema_builder.add_i64_field("updateTimestamp", STORED);
        
        let schema = schema_builder.build();
        
        Self {
            schema,
            id_field,
            metadata_name_field,
            doc_type_field,
            owner_name_field,
            category_field,
            tag_field,
            title_field,
            description_field,
            content_field,
            recycled_field,
            exposed_field,
            published_field,
            annotations_field,
            creation_timestamp_field,
            update_timestamp_field,
            permalink_field,
        }
    }
    
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    
    pub fn convert(&self, halo_doc: &HaloDocument) -> TantivyDocument {
        let mut doc = TantivyDocument::default();
        
        doc.add_text(self.id_field, &halo_doc.id);
        doc.add_text(self.metadata_name_field, &halo_doc.metadata_name);
        doc.add_text(self.doc_type_field, &halo_doc.doc_type);
        doc.add_text(self.owner_name_field, &halo_doc.owner_name);
        doc.add_text(self.title_field, &halo_doc.title);
        doc.add_text(self.content_field, &halo_doc.content);
        doc.add_text(self.recycled_field, &halo_doc.recycled.to_string());
        doc.add_text(self.exposed_field, &halo_doc.exposed.to_string());
        doc.add_text(self.published_field, &halo_doc.published.to_string());
        doc.add_text(self.permalink_field, &halo_doc.permalink);
        
        if let Some(description) = &halo_doc.description {
            doc.add_text(self.description_field, description);
        }
        
        if let Some(categories) = &halo_doc.categories {
            for category in categories {
                doc.add_text(self.category_field, category);
            }
        }
        
        if let Some(tags) = &halo_doc.tags {
            for tag in tags {
                doc.add_text(self.tag_field, tag);
            }
        }
        
        if let Some(annotations) = &halo_doc.annotations {
            // 将annotations序列化为JSON字符串存储
            if let Ok(json_str) = serde_json::to_string(annotations) {
                doc.add_text(self.annotations_field, &json_str);
            }
        }
        
        if let Some(creation_ts) = &halo_doc.creation_timestamp {
            doc.add_i64(self.creation_timestamp_field, creation_ts.timestamp_millis());
        }
        
        if let Some(update_ts) = &halo_doc.update_timestamp {
            doc.add_i64(self.update_timestamp_field, update_ts.timestamp_millis());
        }
        
        doc
    }
}

impl Default for HaloDocumentConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// 将Tantivy Document转换为HaloDocument的转换器
pub struct DocumentConverter {
    id_field: Field,
    metadata_name_field: Field,
    doc_type_field: Field,
    owner_name_field: Field,
    category_field: Field,
    tag_field: Field,
    title_field: Field,
    description_field: Field,
    content_field: Field,
    recycled_field: Field,
    exposed_field: Field,
    published_field: Field,
    annotations_field: Field,
    creation_timestamp_field: Field,
    update_timestamp_field: Field,
    permalink_field: Field,
}

impl DocumentConverter {
    pub fn new(converter: &HaloDocumentConverter) -> Self {
        Self {
            id_field: converter.id_field,
            metadata_name_field: converter.metadata_name_field,
            doc_type_field: converter.doc_type_field,
            owner_name_field: converter.owner_name_field,
            category_field: converter.category_field,
            tag_field: converter.tag_field,
            title_field: converter.title_field,
            description_field: converter.description_field,
            content_field: converter.content_field,
            recycled_field: converter.recycled_field,
            exposed_field: converter.exposed_field,
            published_field: converter.published_field,
            annotations_field: converter.annotations_field,
            creation_timestamp_field: converter.creation_timestamp_field,
            update_timestamp_field: converter.update_timestamp_field,
            permalink_field: converter.permalink_field,
        }
    }
    
    pub fn convert(&self, doc: &TantivyDocument) -> HaloDocument {
        let id = doc.get_first(self.id_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let metadata_name = doc.get_first(self.metadata_name_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let doc_type = doc.get_first(self.doc_type_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let owner_name = doc.get_first(self.owner_name_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let title = doc.get_first(self.title_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let description = doc.get_first(self.description_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let content = doc.get_first(self.content_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let permalink = doc.get_first(self.permalink_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let recycled = doc.get_first(self.recycled_field)
            .and_then(|v| v.as_str())
            .map(|s| s.parse::<bool>().unwrap_or(false))
            .unwrap_or(false);
        
        let exposed = doc.get_first(self.exposed_field)
            .and_then(|v| v.as_str())
            .map(|s| s.parse::<bool>().unwrap_or(false))
            .unwrap_or(false);
        
        let published = doc.get_first(self.published_field)
            .and_then(|v| v.as_str())
            .map(|s| s.parse::<bool>().unwrap_or(false))
            .unwrap_or(false);
        
        let categories: Vec<String> = doc.get_all(self.category_field)
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        
        let tags: Vec<String> = doc.get_all(self.tag_field)
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        
        let annotations = doc.get_first(self.annotations_field)
            .and_then(|v| v.as_str())
            .and_then(|json_str| serde_json::from_str::<HashMap<String, String>>(json_str).ok());
        
        let creation_timestamp = doc.get_first(self.creation_timestamp_field)
            .and_then(|v| {
                v.as_i64().and_then(|ts| DateTime::from_timestamp_millis(ts))
            });
        
        let update_timestamp = doc.get_first(self.update_timestamp_field)
            .and_then(|v| {
                v.as_i64().and_then(|ts| DateTime::from_timestamp_millis(ts))
            });
        
        HaloDocument {
            id,
            metadata_name,
            annotations: if annotations.as_ref().map(|m| m.is_empty()).unwrap_or(true) {
                None
            } else {
                annotations
            },
            title,
            description: if description.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                None
            } else {
                description
            },
            content,
            categories: if categories.is_empty() { None } else { Some(categories) },
            tags: if tags.is_empty() { None } else { Some(tags) },
            published,
            recycled,
            exposed,
            owner_name,
            creation_timestamp,
            update_timestamp,
            permalink,
            doc_type,
        }
    }
}

