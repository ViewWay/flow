# Flow

Flow 是 Halo 项目的 Rust 实现版本，一个强大易用的开源建站工具。

## 项目状态

🚧 **开发中** - 当前已完成核心基础设施和主要功能模块

### 已完成功能 ✅

- ✅ **项目基础设施** - 完整的Rust workspace项目结构
- ✅ **数据库层** - 支持MySQL/PostgreSQL/Redis/MongoDB，Sea-ORM集成
- ✅ **扩展系统** - 完整的Extension系统，包括索引和查询引擎
- ✅ **认证授权** - JWT、Session、RBAC权限控制、多种认证方式
- ✅ **用户管理** - 用户CRUD、角色管理、角色绑定
- ✅ **内容管理** - Post、SinglePage、Comment、Snapshot、Category、Tag完整实现
- ✅ **API路由** - Console端点、UC端点、Extension端点
- ✅ **OpenAPI文档** - 基础框架已实现，SwaggerUI集成完成

### 待实现 📋

- ✅ **全文搜索** - Tantivy集成，支持高亮、排序、缓存
- ✅ **主题系统** - 模板引擎和主题管理
- ✅ **附件管理** - 文件上传和存储，Policy和Group支持，共享URL
- ✅ **WebSocket** - 实时通信支持，插件WebSocket端点
- ✅ **通知系统** - 通知中心实现
- ✅ **备份恢复** - 数据备份和恢复功能
- 📋 **插件系统** - FFI桥接和Rust插件SDK

## 特性

- 🚀 **高性能**: 基于 Rust 和 Tokio 异步运行时
- 🔒 **安全**: 多层安全防护，输入验证，SQL注入防护
- 🗄️ **多数据库支持**: MySQL, PostgreSQL, Redis, MongoDB
- 🔌 **插件系统**: FFI桥接支持Java插件（计划中），逐步迁移到Rust插件
- 📝 **API兼容**: 完全兼容Halo REST API
- 🎯 **类型安全**: Rust的类型系统确保运行时安全

## 技术栈

- **Web框架**: Axum 0.7
- **ORM**: Sea-ORM 0.12
- **数据库**: MySQL, PostgreSQL, Redis, MongoDB
- **全文搜索**: Tantivy（计划中）
- **模板引擎**: Askama + Tera（计划中）
- **WebSocket**: tokio-tungstenite（计划中）
- **OpenAPI**: utoipa + utoipa-swagger-ui

## 快速开始

### 前置要求

- Rust 1.70+
- MySQL 或 PostgreSQL
- Redis
- MongoDB（可选，用于日志）

### 安装

```bash
# 克隆项目
git clone <repository-url>
cd flow

# 构建项目
cargo build --release

# 运行
cargo run
```

### 配置

复制 `flow.toml` 到工作目录 `~/.flow2/flow.toml` 并修改配置。

配置文件示例：

```toml
[server]
port = 8090
host = "0.0.0.0"

[database.mysql]
url = "mysql://user:password@localhost:3306/flow"

[redis]
url = "redis://localhost:6379"

[mongodb]
url = "mongodb://localhost:27017"
```

## 项目结构

```
flow/
├── flow/              # 主应用模块
│   ├── src/
│   │   ├── main.rs    # 应用入口
│   │   ├── config.rs  # 配置管理
│   │   ├── server.rs  # HTTP服务器和路由
│   │   └── error.rs   # 错误处理
│   └── Cargo.toml
├── flow-api/          # API定义模块
│   └── src/
│       ├── extension/ # 扩展系统API
│       └── security/  # 安全相关API
├── flow-domain/       # 领域模型模块
│   └── src/
│       ├── content/   # 内容领域模型
│       └── security/  # 安全领域模型
├── flow-infra/        # 基础设施模块
│   └── src/
│       ├── database/  # 数据库连接和Repository
│       ├── cache/     # 缓存实现
│       ├── index/     # 索引系统
│       └── security/  # 安全基础设施
├── flow-service/      # 服务层模块
│   └── src/
│       ├── content/   # 内容服务
│       └── security/  # 安全服务
├── flow-web/          # Web层模块
│   └── src/
│       ├── handlers/  # 请求处理器
│       ├── security/  # 安全中间件
│       └── openapi.rs # OpenAPI文档
├── flow-plugin/       # 插件系统模块（计划中）
└── flow-migration/    # 数据库迁移模块
```

## API端点

### Console端点 (`/api/v1alpha1/*`)

- `GET/POST /api/v1alpha1/posts` - 文章管理
- `GET/POST /api/v1alpha1/users` - 用户管理
- `GET/POST /api/v1alpha1/roles` - 角色管理
- `GET/POST /api/v1alpha1/comments` - 评论管理
- `GET/POST /api/v1alpha1/categories` - 分类管理
- `GET/POST /api/v1alpha1/tags` - 标签管理

### UC端点 (`/api/v1alpha1/uc/*`)

- `GET/POST /api/v1alpha1/uc/posts` - 用户自己的文章管理
- `GET /api/v1alpha1/uc/posts/{name}` - 获取用户文章
- `PUT /api/v1alpha1/uc/posts/{name}/publish` - 发布文章

### Extension端点 (`/apis/{group}/{version}/{resource}`)

- `GET /apis/{group}/{version}/{resource}` - 列出扩展对象
- `GET /apis/{group}/{version}/{resource}/{name}` - 获取扩展对象
- `POST /apis/{group}/{version}/{resource}` - 创建扩展对象
- `PUT /apis/{group}/{version}/{resource}/{name}` - 更新扩展对象
- `DELETE /apis/{group}/{version}/{resource}/{name}` - 删除扩展对象

## 开发

```bash
# 运行测试
cargo test

# 运行开发服务器
cargo run

# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 构建文档
cargo doc --open
```

## 开发进度

**总体进度**: 13/17阶段已完成（约76%）

### 阶段1: 项目基础设施 ✅ 100%
- [x] Rust workspace项目结构
- [x] 配置管理系统
- [x] 错误处理系统
- [x] 日志系统

### 阶段2: 数据库层 ✅ 100%
- [x] DatabaseManager实现
- [x] ExtensionStore实体和Repository
- [x] Sea-ORM迁移
- [x] Redis缓存抽象
- [x] MongoDB连接

### 阶段3: 扩展系统 ✅ 100%
- [x] Extension trait和Metadata
- [x] ExtensionClient实现
- [x] 索引系统（LabelIndex, SingleValueIndex, MultiValueIndex）
- [x] 查询引擎（IndexedQueryEngine）

### 阶段4: 认证授权 ✅ 100%
- [x] JWT令牌生成和验证
- [x] Session管理
- [x] 认证中间件
- [x] 授权管理器（RBAC）
- [x] PAT支持

### 阶段5: 用户和权限管理 ✅ 100%
- [x] User实体和服务
- [x] Role和RoleBinding
- [x] 用户CRUD操作
- [x] 权限检查逻辑

### 阶段6: 内容管理 ✅ 100%
- [x] Post实体和服务
- [x] SinglePage实体和服务
- [x] Comment实体和服务
- [x] Snapshot版本控制
- [x] Category和Tag
- [x] 内容查询和过滤

### 阶段7: API路由 ✅ 95%
- [x] Axum路由结构
- [x] Console端点
- [x] UC端点
- [x] Extension端点
- [x] OpenAPI文档基础框架
- [ ] SwaggerUI集成（待完善）

### 阶段8: 全文搜索实现 ✅ 100%
- [x] 集成Tantivy
- [x] 实现搜索索引构建
- [x] 实现文档索引和更新
- [x] 实现搜索服务
- [x] SearchEngine 集成到 Contains 查询（架构改动完成）
- [x] 使用 Tantivy 原生高亮功能优化搜索结果高亮
- [x] 实现搜索API端点
- [x] 完整测试和文档

### 阶段9: 主题系统 ✅ 100%
- [x] Theme实体和服务
- [x] ThemeService实现
- [x] 主题安装和升级功能
- [x] 主题解析器（ThemeResolver）
- [x] 模板引擎集成（Tera）
- [x] 主题静态资源服务
- [x] 主题API端点（list、get、activate、install、upgrade、reload）
- [x] 系统设置服务（主题设置管理）
- [x] 主题模板渲染（render_theme_template）
- [x] 主题路由处理器（post_page、category_page、tag_page、archive_page）
- [x] Finder完善（PostFinder、CategoryFinder、TagFinder、ThemeFinder）
### 阶段10: 附件管理 ✅ 100%
- [x] Attachment实体和服务
- [x] AttachmentService实现
- [x] 附件存储抽象（AttachmentStorage）
- [x] 本地存储实现（LocalAttachmentStorage）
- [x] 缩略图服务（ThumbnailService）
- [x] 附件上传API端点（支持multipart/form-data，支持policyName和groupName）
- [x] 附件查询和过滤（按groupName、policyName、ownerName、tag）
- [x] 附件CRUD操作
- [x] Policy实体和服务（PolicyService）
- [x] PolicyTemplate实体和服务（PolicyTemplateService）
- [x] Group实体和服务（GroupService）
- [x] Group附件计数功能
- [x] Policy和Group的API端点
- [x] 共享URL功能（SharedUrlService）
- [x] 共享URL生成、验证、撤销和访问
### 阶段11: WebSocket支持 ✅ 100%
- [x] WebSocketEndpoint trait定义
- [x] WebSocketEndpointManager实现
- [x] WebSocket连接处理器
- [x] WebSocket路由集成（/apis路径）
- [x] Echo示例端点
- [x] WebSocket权限检查（认证和授权）
### 阶段12: 通知系统 ✅ 100%
- [x] Notification实体和服务（NotificationService）
- [x] NotificationTemplate实体和服务
- [x] Reason实体和服务
- [x] Subscription实体和服务
- [x] NotificationCenter实现（通知发送和管理）
- [x] NotificationSender trait（通知发送器接口）
- [x] 通知API端点（CRUD、标记已读、未读数量）
- [x] 订阅匹配逻辑（reason_type和subject匹配）
- [x] Subscription和Reason的API端点
- [x] 通过token取消订阅功能
### 阶段13: 备份恢复系统 ✅ 100%
- [x] Backup实体和服务（BackupService）
- [x] RestoreService实现
- [x] 备份文件管理（创建、下载、删除、列表）
- [x] 备份API端点（create、download、delete、list）
- [x] 恢复API端点（restore）
- [x] 扩展数据备份和恢复
- [x] 工作目录备份和恢复
- [x] ZIP格式备份文件打包和解压
- [ ] 插件系统（FFI桥接和Rust插件SDK）
- [ ] API兼容性测试
- [ ] 集成测试和优化
- [ ] 文档和部署

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](../CONTRIBUTING.md) 了解详细信息。

## 许可证

GPL-3.0

## 相关链接

- [Halo 原项目](https://github.com/halo-dev/halo)
- [Halo 文档](https://docs.halo.run)

## 致谢

本项目基于 [Halo](https://github.com/halo-dev/halo) 项目，使用 Rust 重新实现，旨在提供更高的性能和更好的类型安全。
