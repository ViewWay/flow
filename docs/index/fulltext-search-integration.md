# 全文搜索集成到 Contains 查询

## 概述

本文档说明如何将 `SearchEngine` 集成到 `Contains` 查询中，使得对于全文搜索字段（如 `spec.title`、`status.excerpt`）可以使用全文搜索引擎进行搜索，而不是简单的字符串匹配。

## 架构设计

### 核心组件

1. **QueryVisitor** (`flow-infra/src/index/query_visitor.rs`)
   - 查询条件遍历器
   - 支持同步和异步查询
   - 集成 `SearchEngine` 进行全文搜索

2. **SearchEngine** (`flow-api/src/search/mod.rs`)
   - 全文搜索引擎接口
   - 支持 Tantivy 等实现

3. **FulltextFieldMapping** (`flow-infra/src/index/fulltext_field_mapping.rs`)
   - 定义哪些索引字段应该使用全文搜索
   - 映射关系：`index_name -> doc_type` 列表

4. **DocTypeProvider** (`flow-infra/src/index/doc_type_converter.rs`)
   - 为 Extension 类型提供文档类型转换
   - 文档类型格式：`{kind.lowercase()}.{group}`

5. **IndexEngine** (`flow-infra/src/index/engine.rs`)
   - 索引引擎接口
   - 支持全文搜索的查询方法

### 工作流程

```
Contains 查询
    ↓
检查字段是否是全文搜索字段 (FulltextFieldMapping)
    ↓
是 → 检查 SearchEngine 是否可用
    ↓
可用 → 获取文档类型 (DocTypeProvider)
    ↓
构建 SearchOption → 执行全文搜索
    ↓
提取 metadata_name 集合 → 返回结果
    ↓
不可用/失败 → 回退到字符串匹配
```

## 使用方法

### 1. 为 Extension 类型实现 DocTypeProvider

```rust
use flow_infra::index::DocTypeProvider;
use flow_domain::content::Post;

impl DocTypeProvider for Post {
    fn doc_type() -> String {
        "post.content.halo.run".to_string()
    }
}
```

### 2. 配置全文搜索字段映射

`FulltextFieldMapping` 默认配置了以下字段：

- `spec.title` → `["post.content.halo.run"]`
- `status.excerpt` → `["post.content.halo.run"]`
- `spec.title` → `["singlepage.content.halo.run"]`

可以通过修改 `FulltextFieldMapping::default()` 来添加更多字段映射。

### 3. 创建 IndexEngine 并注入依赖

```rust
use flow_infra::index::{IndexEngine, DefaultIndexEngine, IndicesManager, FulltextFieldMapping};

let indices_manager = Arc::new(IndicesManager::new());
let fulltext_mapping = Arc::new(FulltextFieldMapping::default());
let search_engine: Arc<dyn SearchEngine> = /* ... */;

let index_engine: Arc<dyn IndexEngine> = Arc::new(
    DefaultIndexEngine::with_search_engine(
        indices_manager,
        Some(search_engine),
        fulltext_mapping,
    )
);
```

### 4. 使用全文搜索查询

对于实现了 `DocTypeProvider` 的类型，可以使用 `retrieve_with_fulltext()` 方法：

```rust
use flow_api::extension::ListOptions;
use flow_api::extension::query::Condition;

let options = ListOptions {
    condition: Some(Condition::Contains {
        index_name: "spec.title".to_string(),
        value: "关键词".to_string(),
    }),
    ..Default::default()
};

// 使用全文搜索版本
let result = index_engine.retrieve_with_fulltext::<Post>(&options).await?;
```

对于没有实现 `DocTypeProvider` 的类型，可以使用标准的 `retrieve()` 方法，会自动回退到字符串匹配。

## 回退机制

系统实现了完善的回退机制：

1. **SearchEngine 不可用**：回退到字符串匹配
2. **搜索失败**：回退到字符串匹配
3. **类型未实现 DocTypeProvider**：回退到字符串匹配
4. **字段不是全文搜索字段**：使用字符串匹配

所有回退操作都会记录警告日志，便于调试和监控。

## 性能考虑

1. **搜索限制**：默认设置 `limit: 10000`，确保获取所有匹配结果
2. **文档类型过滤**：通过 `include_types` 过滤，只搜索当前 Extension 类型的文档
3. **异步执行**：所有搜索操作都是异步的，不会阻塞其他查询

## 测试

提供了 `MockSearchEngine` 用于测试：

```rust
use flow_infra::index::query_visitor_test::MockSearchEngine;

let mock_engine = MockSearchEngine::new(true)
    .with_results(vec![halo_document]);
```

## 注意事项

1. **文档类型格式**：必须遵循 `{kind.lowercase()}.{group}` 格式
2. **字段映射**：一个字段可能对应多个文档类型，需要通过 `DocTypeProvider` 过滤
3. **递归查询**：在 AND/OR/NOT 组合查询中，全文搜索功能会传递到子查询
4. **类型安全**：只有实现了 `DocTypeProvider` 的类型才能使用 `retrieve_with_fulltext()`

## 未来改进

1. 支持更多全文搜索字段
2. 优化搜索性能（缓存、分页等）
3. 支持更复杂的搜索选项（高亮、排序等）
4. 提供搜索统计和监控功能

