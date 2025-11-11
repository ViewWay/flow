//! 搜索功能集成测试
//! 
//! 这些测试验证搜索功能的端到端行为，包括：
//! - 文档索引和搜索
//! - 高亮功能
//! - 过滤条件
//! - API端点

#[cfg(test)]
mod integration_tests {
    use flow_api::search::{HaloDocument, SearchOption};
    use crate::search::TantivySearchEngine;
    use flow_service::search::{SearchService, DefaultSearchService};
    use chrono::Utc;
    use tempfile::TempDir;
    use std::sync::Arc;

    /// 创建测试用的搜索服务
    async fn create_test_search_service() -> (TempDir, Arc<dyn SearchService>) {
        let temp_dir = TempDir::new().unwrap();
        let engine = Arc::new(
            TantivySearchEngine::new(temp_dir.path()).await.unwrap()
        );
        let service: Arc<dyn SearchService> = Arc::new(
            DefaultSearchService::new(engine)
        );
        (temp_dir, service)
    }

    /// 创建测试文档
    fn create_test_document(
        id: &str,
        title: &str,
        description: Option<&str>,
        content: &str,
        published: bool,
        exposed: bool,
    ) -> HaloDocument {
        HaloDocument {
            id: format!("post.content.halo.run-{}", id),
            metadata_name: id.to_string(),
            annotations: None,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            content: content.to_string(),
            categories: None,
            tags: None,
            published,
            recycled: false,
            exposed,
            owner_name: "admin".to_string(),
            creation_timestamp: Some(Utc::now()),
            update_timestamp: Some(Utc::now()),
            permalink: format!("/{}", id),
            doc_type: "post.content.halo.run".to_string(),
        }
    }

    /// 测试：基本搜索功能
    #[tokio::test]
    async fn test_basic_search() {
        let (_temp_dir, service) = create_test_search_service().await;
        
        // 添加测试文档
        let doc1 = create_test_document(
            "test-1",
            "Rust Programming Guide",
            Some("Learn Rust programming"),
            "Rust is a systems programming language.",
            true,
            true,
        );
        let doc2 = create_test_document(
            "test-2",
            "Python Tutorial",
            Some("Learn Python"),
            "Python is a high-level programming language.",
            true,
            true,
        );
        
        service.add_or_update(vec![doc1, doc2]).await.unwrap();
        
        // 搜索 "Rust"
        let option = SearchOption {
            keyword: "Rust".to_string(),
            limit: 10,
            highlight_pre_tag: "<mark>".to_string(),
            highlight_post_tag: "</mark>".to_string(),
            filter_exposed: None,
            filter_recycled: None,
            filter_published: None,
            include_types: None,
            include_owner_names: None,
            include_category_names: None,
            include_tag_names: None,
            sort_by: None,
            sort_order: flow_api::search::SortOrder::Desc,
            annotations: None,
        };
        
        let result = service.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].metadata_name, "test-1");
        assert!(result.hits[0].title.contains("Rust"));
    }

    /// 测试：高亮功能集成
    #[tokio::test]
    async fn test_highlighting_integration() {
        let (_temp_dir, service) = create_test_search_service().await;
        
        let doc = create_test_document(
            "highlight-test",
            "Rust Programming Language",
            Some("Rust is a modern language"),
            "Rust provides memory safety without garbage collection.",
            true,
            true,
        );
        
        service.add_or_update(vec![doc]).await.unwrap();
        
        let option = SearchOption {
            keyword: "Rust".to_string(),
            limit: 10,
            highlight_pre_tag: "<B>".to_string(),
            highlight_post_tag: "</B>".to_string(),
            filter_exposed: None,
            filter_recycled: None,
            filter_published: None,
            include_types: None,
            include_owner_names: None,
            include_category_names: None,
            include_tag_names: None,
            sort_by: None,
            sort_order: flow_api::search::SortOrder::Desc,
            annotations: None,
        };
        
        let result = service.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        
        let hit = &result.hits[0];
        // 验证高亮标签存在
        assert!(hit.title.contains("<B>"));
        assert!(hit.title.contains("</B>"));
        assert!(hit.content.contains("<B>"));
        if let Some(ref desc) = hit.description {
            assert!(desc.contains("<B>"));
        }
    }

    /// 测试：过滤已发布内容
    #[tokio::test]
    async fn test_filter_published() {
        let (_temp_dir, service) = create_test_search_service().await;
        
        let doc1 = create_test_document(
            "published-1",
            "Published Post",
            None,
            "This is published",
            true,
            true,
        );
        let doc2 = create_test_document(
            "draft-1",
            "Draft Post",
            None,
            "This is a draft",
            false,
            true,
        );
        
        service.add_or_update(vec![doc1, doc2]).await.unwrap();
        
        // 只搜索已发布的内容
        let option = SearchOption {
            keyword: "Post".to_string(),
            limit: 10,
            highlight_pre_tag: "<B>".to_string(),
            highlight_post_tag: "</B>".to_string(),
            filter_exposed: None,
            filter_recycled: None,
            filter_published: Some(true),
            include_types: None,
            include_owner_names: None,
            include_category_names: None,
            include_tag_names: None,
            sort_by: None,
            sort_order: flow_api::search::SortOrder::Desc,
            annotations: None,
        };
        
        let result = service.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].metadata_name, "published-1");
    }

    /// 测试：过滤公开内容
    #[tokio::test]
    async fn test_filter_exposed() {
        let (_temp_dir, service) = create_test_search_service().await;
        
        let doc1 = create_test_document(
            "public-1",
            "Public Post",
            None,
            "This is public",
            true,
            true,
        );
        let doc2 = create_test_document(
            "private-1",
            "Private Post",
            None,
            "This is private",
            true,
            false,
        );
        
        service.add_or_update(vec![doc1, doc2]).await.unwrap();
        
        // 只搜索公开的内容
        let option = SearchOption {
            keyword: "Post".to_string(),
            limit: 10,
            highlight_pre_tag: "<B>".to_string(),
            highlight_post_tag: "</B>".to_string(),
            filter_exposed: Some(true),
            filter_recycled: None,
            filter_published: None,
            include_types: None,
            include_owner_names: None,
            include_category_names: None,
            include_tag_names: None,
            sort_by: None,
            sort_order: flow_api::search::SortOrder::Desc,
            annotations: None,
        };
        
        let result = service.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].metadata_name, "public-1");
    }

    /// 测试：按文档类型过滤
    #[tokio::test]
    async fn test_filter_by_type() {
        let (_temp_dir, service) = create_test_search_service().await;
        
        let doc1 = create_test_document(
            "post-1",
            "Post Title",
            None,
            "Post content",
            true,
            true,
        );
        let mut doc2 = create_test_document(
            "page-1",
            "Page Title",
            None,
            "Page content",
            true,
            true,
        );
        doc2.doc_type = "singlepage.content.halo.run".to_string();
        doc2.id = format!("singlepage.content.halo.run-{}", doc2.metadata_name);
        
        service.add_or_update(vec![doc1, doc2]).await.unwrap();
        
        // 只搜索 post 类型
        let option = SearchOption {
            keyword: "Title".to_string(),
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
        
        let result = service.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].metadata_name, "post-1");
    }

    /// 测试：文档更新
    #[tokio::test]
    async fn test_document_update() {
        let (_temp_dir, service) = create_test_search_service().await;
        
        let mut doc = create_test_document(
            "update-test",
            "Original Title",
            None,
            "Original content",
            true,
            true,
        );
        
        service.add_or_update(vec![doc.clone()]).await.unwrap();
        
        // 更新文档
        doc.title = "Updated Title".to_string();
        doc.content = "Updated content".to_string();
        service.add_or_update(vec![doc]).await.unwrap();
        
        // 搜索更新后的内容
        let option = SearchOption {
            keyword: "Updated".to_string(),
            limit: 10,
            highlight_pre_tag: "<B>".to_string(),
            highlight_post_tag: "</B>".to_string(),
            filter_exposed: None,
            filter_recycled: None,
            filter_published: None,
            include_types: None,
            include_owner_names: None,
            include_category_names: None,
            include_tag_names: None,
            sort_by: None,
            sort_order: flow_api::search::SortOrder::Desc,
            annotations: None,
        };
        
        let result = service.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].title, "Updated Title");
    }

    /// 测试：文档删除
    #[tokio::test]
    async fn test_document_deletion() {
        let (_temp_dir, service) = create_test_search_service().await;
        
        let doc = create_test_document(
            "delete-test",
            "To Be Deleted",
            None,
            "This will be deleted",
            true,
            true,
        );
        
        service.add_or_update(vec![doc]).await.unwrap();
        
        // 验证文档存在
        let option = SearchOption {
            keyword: "Deleted".to_string(),
            limit: 10,
            highlight_pre_tag: "<B>".to_string(),
            highlight_post_tag: "</B>".to_string(),
            filter_exposed: None,
            filter_recycled: None,
            filter_published: None,
            include_types: None,
            include_owner_names: None,
            include_category_names: None,
            include_tag_names: None,
            sort_by: None,
            sort_order: flow_api::search::SortOrder::Desc,
            annotations: None,
        };
        
        let result = service.search(option.clone()).await.unwrap();
        assert_eq!(result.hits.len(), 1);
        
        // 删除文档
        service.delete_document(vec!["post.content.halo.run-delete-test".to_string()]).await.unwrap();
        
        // 验证文档已删除
        let result = service.search(option).await.unwrap();
        assert_eq!(result.hits.len(), 0);
    }
}

