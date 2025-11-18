# Flow 实现状态对比文档

本文档对比了 Flow 项目与 Halo 原项目文档的实现完成情况。

## 认证系统 (authentication/README.md)

### ✅ 已完成

- ✅ **Basic Auth** - 基本认证支持
- ✅ **Form Login** - 表单登录支持
- ✅ **PAT (Personal Access Token)** - 个人访问令牌
  - PAT实体定义（PersonalAccessToken）
  - PatProvider认证提供者
  - JWT格式的PAT token（pat_前缀 + JWT）
  - Token验证和角色绑定
- ✅ **OAuth2** - OAuth2认证
  - 授权码流程
  - CSRF保护（state token）
  - Token缓存
  - 已登录用户绑定
  - 从ConfigMap读取OAuth2配置
- ✅ **2FA/TOTP** - 双因素认证
  - TOTP代码生成和验证
  - AES-GCM加密存储TOTP密钥
  - Session状态管理
  - 配置化issuer支持

### 📋 待实现

- 📋 **Remember Me** - 记住我功能（文档中提到计划支持）

## 通知系统 (notification/README.md)

### ✅ 已完成

- ✅ **Reason** - 通知原因实体
- ✅ **ReasonType** - 通知原因类型（概念上支持）
- ✅ **Subscription** - 订阅实体
  - 支持reason_type和subject匹配
  - 支持SpEL表达式匹配（使用evalexpr库）
- ✅ **Notification** - 站内通知实体
- ✅ **NotificationTemplate** - 通知模板实体
  - 模板查找和选择逻辑
  - 语言优先级匹配
- ✅ **NotificationCenter** - 通知中心实现
  - 通知发送流程
  - 订阅匹配逻辑
  - 模板渲染
- ✅ **NotificationSender** - 通知发送器trait
  - 基础实现（InMemoryNotificationSender）
  - 支持扩展邮件、短信等通知方式
- ✅ **通知API端点**
  - CRUD操作
  - 标记已读/未读
  - 未读数量查询

### 📋 待实现

- 📋 **NotifierDescriptor** - 通知器描述符实体
  - 用于声明通知器扩展
  - 关联ExtensionDefinition
- 📋 **用户通知偏好设置** - 从ConfigMap读取用户偏好
  - ConfigMap格式：`user-preferences-{username}`
  - 存储reasonType与notifier的映射关系
- 📋 **个人中心通知API** - userspaces路径
  - `GET /apis/api.notification.halo.run/v1alpha1/userspaces/{username}/notifications`
  - `PUT /apis/api.notification.halo.run/v1alpha1/userspaces/{username}/notifications/mark-as-read`
  - `PUT /apis/api.notification.halo.run/v1alpha1/userspaces/{username}/notifications/mark-specified-as-read`
- 📋 **Notifier配置API**
  - `GET /apis/api.console.halo.run/v1alpha1/notifiers/{name}/sender-config`
  - `POST /apis/api.console.halo.run/v1alpha1/notifiers/{name}/sender-config`
  - `GET /apis/api.notification.halo.run/v1alpha1/notifiers/{name}/receiver-config`
  - `POST /apis/api.notification.halo.run/v1alpha1/notifiers/{name}/receiver-config`
- 📋 **通知模板渲染** - ThymeleafEngine支持
  - 当前使用简单的字符串替换
  - 需要支持Thymeleaf模板语法

## 备份恢复系统 (backup-and-restore.md)

### ✅ 已完成

- ✅ **Backup实体和服务** - BackupService实现
- ✅ **RestoreService** - 恢复服务实现
- ✅ **备份文件管理**
  - 创建备份
  - 下载备份
  - 删除备份
  - 列表查询
- ✅ **备份API端点**
  - `POST /apis/migration.halo.run/v1alpha1/backups` - 创建备份
  - `GET /apis/migration.halo.run/v1alpha1/backups` - 列表备份
  - `GET /apis/migration.halo.run/v1alpha1/backups/{name}` - 获取备份
  - `GET /apis/migration.halo.run/v1alpha1/backups/{name}/download` - 下载备份
  - `DELETE /apis/migration.halo.run/v1alpha1/backups/{name}` - 删除备份
- ✅ **恢复API端点**
  - `POST /apis/migration.halo.run/v1alpha1/restorations` - 恢复备份
- ✅ **扩展数据备份和恢复** - ExtensionStore数据备份
- ✅ **工作目录备份和恢复** - themes、attachments、keys等目录备份
- ✅ **ZIP格式备份文件打包和解压**

### 📋 待实现

- 📋 **备份文件摘要校验** - 计算和验证备份文件完整性
- 📋 **异步备份执行** - BackupReconciler模式（当前是同步执行）
- 📋 **备份状态管理** - phase字段（Pending/Running/Succeeded/Failed）
- 📋 **备份配置** - config.yaml文件（描述压缩格式）

## 其他文档

### 扩展点 (extension-points/)

- ✅ **认证扩展点** - AuthenticationProvider trait支持
- ✅ **内容扩展点** - PostContentHandler概念（需要插件系统支持）
- 📋 **搜索引擎扩展点** - 需要插件系统支持

### 开发者指南 (developer-guide/)

- ✅ **自定义端点** - Extension端点支持
- 📋 **插件配置属性** - 需要插件系统支持

### WebSocket (plugin/websocket.md)

- ✅ **WebSocket支持** - WebSocketEndpoint trait和Manager
- ✅ **认证和授权** - WebSocket连接认证
- ✅ **动态端点注册** - 支持插件注册WebSocket端点

## 总体完成度

### 认证系统: ~95%
- 核心功能已完成
- 缺少Remember Me功能

### 通知系统: ~75%
- 核心功能已完成
- 缺少NotifierDescriptor和用户偏好设置
- 缺少userspaces API路径
- 缺少Notifier配置API

### 备份恢复系统: ~90%
- 核心功能已完成
- 缺少异步执行和状态管理
- 缺少备份文件摘要校验

### 总体进度: ~85%

## 下一步建议

1. **实现NotifierDescriptor实体** - 完善通知系统
2. **实现用户通知偏好设置** - 从ConfigMap读取用户偏好
3. **实现userspaces API路径** - 个人中心通知API
4. **实现备份异步执行** - BackupReconciler模式
5. **实现备份文件摘要校验** - 确保备份文件完整性

