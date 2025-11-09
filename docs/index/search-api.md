# 搜索API文档

## 概述

搜索API提供了全文搜索功能，支持关键词搜索、过滤条件和高亮显示。搜索功能基于 Tantivy 搜索引擎实现，提供了高性能和准确的搜索结果。

## 端点

### GET /api/v1alpha1/search

执行搜索查询并返回匹配的文档列表。

#### 请求参数

所有参数都是查询参数（Query Parameters）：

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
|--------|------|------|--------|------|
| `keyword` | string | 是 | - | 搜索关键词 |
| `limit` | integer | 否 | 10 | 返回结果的最大数量（1-1000） |
| `highlightPreTag` | string | 否 | `<B>` | 高亮片段的前HTML标签 |
| `highlightPostTag` | string | 否 | `</B>` | 高亮片段的后HTML标签 |
| `filterExposed` | boolean | 否 | null | 是否只搜索公开内容（true/false/null） |
| `filterRecycled` | boolean | 否 | null | 是否过滤回收站内容（true/false/null） |
| `filterPublished` | boolean | 否 | null | 是否只搜索已发布内容（true/false/null） |
| `includeTypes` | string[] | 否 | null | 要包含的文档类型列表（OR关系） |
| `includeOwnerNames` | string[] | 否 | null | 要包含的所有者名称列表（OR关系） |
| `includeCategoryNames` | string[] | 否 | null | 要包含的分类名称列表（AND关系） |
| `includeTagNames` | string[] | 否 | null | 要包含的标签名称列表（AND关系） |
| `sortBy` | string | 否 | null | 排序字段：`relevance`（相关性，默认）、`creationTime`（创建时间）、`updateTime`（更新时间）、`title`（标题） |
| `sortOrder` | string | 否 | `desc` | 排序方向：`asc`（升序）、`desc`（降序） |

#### 排序功能

搜索API支持多种排序方式：

- **相关性排序**（默认）：使用 `sortBy=relevance` 或不指定 `sortBy` 参数，结果按搜索相关性排序
- **创建时间排序**：使用 `sortBy=creationTime`，按文档创建时间排序
- **更新时间排序**：使用 `sortBy=updateTime`，按文档更新时间排序
- **标题排序**：使用 `sortBy=title`，按文档标题字母顺序排序

排序方向通过 `sortOrder` 参数控制：
- `asc`：升序（从小到大）
- `desc`：降序（从大到小，默认）

**示例**：

按更新时间降序排序：
```
GET /api/v1alpha1/search?keyword=Rust&sortBy=updateTime&sortOrder=desc
```

按标题升序排序：
```
GET /api/v1alpha1/search?keyword=Rust&sortBy=title&sortOrder=asc
```

#### 高亮功能

搜索API支持使用 Tantivy 原生高亮功能，可以自定义高亮标签。高亮功能会自动应用到以下字段：

- `title` - 文档标题
- `description` - 文档描述（如果存在）
- `content` - 文档内容

**高亮标签说明**：

- `highlightPreTag`: 高亮片段的前HTML标签，默认值为 `<B>`
- `highlightPostTag`: 高亮片段的后HTML标签，默认值为 `</B>`

**示例**：

使用默认高亮标签（`<B>` 和 `</B>`）：
```
GET /api/v1alpha1/search?keyword=Rust&limit=10
```

使用自定义高亮标签：
```
GET /api/v1alpha1/search?keyword=Rust&highlightPreTag=<mark>&highlightPostTag=</mark>
```

使用CSS类名：
```
GET /api/v1alpha1/search?keyword=Rust&highlightPreTag=<span class="highlight">&highlightPostTag=</span>
```

#### 响应格式

```json
{
  "hits": [
    {
      "id": "post.content.halo.run-my-post",
      "metadata_name": "my-post",
      "title": "Rust <B>Programming</B> Guide",
      "description": "Learn <B>Rust</B> programming language",
      "content": "This is a guide about <B>Rust</B> programming...",
      "permalink": "/my-post",
      "doc_type": "post.content.halo.run",
      "published": true,
      "exposed": true,
      "recycled": false,
      "owner_name": "admin",
      "creation_timestamp": "2025-01-01T00:00:00Z",
      "update_timestamp": "2025-01-01T00:00:00Z",
      "categories": null,
      "tags": null,
      "annotations": null
    }
  ],
  "keyword": "Rust",
  "total": 1,
  "limit": 10,
  "processing_time_millis": 5,
  "from_cache": false,
  "cache_stats": null
}
```

#### 响应字段说明

- `hits`: 搜索结果列表，每个结果包含：
  - `title`, `description`, `content`: 已应用高亮的字段，匹配的关键词会被 `highlightPreTag` 和 `highlightPostTag` 包裹
  - 其他字段：文档的元数据信息
- `keyword`: 搜索关键词
- `total`: 总结果数
- `limit`: 结果限制
- `processing_time_millis`: 处理时间（毫秒）
- `from_cache`: 是否来自缓存（如果启用了缓存）
- `cache_stats`: 缓存统计信息（如果启用了缓存），包含：
  - `hits`: 缓存命中次数
  - `misses`: 缓存未命中次数
  - `size`: 缓存大小

#### 错误响应

**400 Bad Request** - 关键词为空或limit超出范围：
```json
{
  "status": "error",
  "message": "Keyword cannot be empty"
}
```

**500 Internal Server Error** - 搜索失败：
```json
{
  "status": "error",
  "message": "Search failed: <error details>"
}
```

## 使用示例

### 基本搜索

```bash
curl "http://localhost:8090/api/v1alpha1/search?keyword=Rust&limit=10"
```

### 使用自定义高亮标签

```bash
curl "http://localhost:8090/api/v1alpha1/search?keyword=Rust&highlightPreTag=<mark>&highlightPostTag=</mark>"
```

### 搜索已发布的公开内容

```bash
curl "http://localhost:8090/api/v1alpha1/search?keyword=Rust&filterPublished=true&filterExposed=true"
```

### 按文档类型过滤

```bash
curl "http://localhost:8090/api/v1alpha1/search?keyword=Rust&includeTypes=post.content.halo.run&includeTypes=singlepage.content.halo.run"
```

### 按分类和标签过滤

```bash
curl "http://localhost:8090/api/v1alpha1/search?keyword=Rust&includeCategoryNames=tech&includeTagNames=programming"
```

### 使用排序功能

按更新时间降序排序：
```bash
curl "http://localhost:8090/api/v1alpha1/search?keyword=Rust&sortBy=updateTime&sortOrder=desc"
```

按标题升序排序：
```bash
curl "http://localhost:8090/api/v1alpha1/search?keyword=Rust&sortBy=title&sortOrder=asc"
```

## 性能优化

### 搜索结果缓存

搜索API支持结果缓存以提高性能。缓存功能通过 `CachedSearchService` 实现：

- **缓存键生成**：使用 SHA256 哈希搜索选项生成唯一的缓存键
- **缓存TTL**：可配置的缓存过期时间
- **缓存统计**：提供缓存命中率等统计信息
- **自动失效**：文档更新或删除时自动清除相关缓存

### 性能监控

搜索服务提供性能监控功能：

- **搜索统计**：总搜索次数、平均搜索时间
- **缓存统计**：缓存命中率、缓存大小
- **性能指标**：处理时间、响应时间

## 高亮功能详解

### 工作原理

搜索API使用 Tantivy 的 `SnippetGenerator` 来实现高亮功能：

1. **分词和匹配**：使用 Tantivy 的分词器对文本进行分词，然后匹配搜索关键词
2. **片段生成**：为每个匹配的字段生成包含高亮信息的片段
3. **标签应用**：根据 `highlightPreTag` 和 `highlightPostTag` 参数，将匹配的文本包裹在高亮标签中
4. **多字段支持**：自动对 `title`、`description` 和 `content` 三个字段应用高亮

### 高亮标签建议

**HTML标签**：
- `<mark>` 和 `</mark>` - HTML5标准高亮标签
- `<strong>` 和 `</strong>` - 强调标签
- `<em>` 和 `</em>` - 斜体标签

**带CSS类的标签**：
- `<span class="highlight">` 和 `</span>` - 使用CSS类控制样式
- `<span class="search-highlight">` 和 `</span>` - 更具体的类名

**示例CSS**：
```css
.highlight {
  background-color: yellow;
  font-weight: bold;
}

.search-highlight {
  background-color: #ffd700;
  padding: 2px 4px;
  border-radius: 2px;
}
```

### 注意事项

1. **空关键词**：如果关键词为空，不会应用高亮功能，但会返回所有文档（如果不过滤）
2. **无匹配结果**：如果没有匹配的文档，返回空的 `hits` 数组
3. **标签转义**：高亮标签会直接插入到文本中，前端需要确保正确渲染HTML
4. **性能考虑**：高亮功能会增加一定的处理时间，但对于大多数查询影响很小

## 技术实现

搜索功能基于以下技术：

- **搜索引擎**：Tantivy 0.25.0
- **高亮实现**：使用 `SnippetGenerator` API
- **分词器**：Tantivy 默认分词器（支持多语言）
- **索引字段**：title、description、content 支持全文搜索

## 相关文档

- [全文搜索集成文档](./fulltext-search-integration.md)
- [索引机制文档](./README.md)

