# Flow

Flow 是 Halo 项目的 Rust 实现版本，一个强大易用的开源建站工具。

## 特性

- 🚀 **高性能**: 基于 Rust 和 Tokio 异步运行时
- 🔒 **安全**: 多层安全防护，输入验证，SQL注入防护
- 🗄️ **多数据库支持**: MySQL, PostgreSQL, Redis, MongoDB
- 🔌 **插件系统**: FFI桥接支持Java插件，逐步迁移到Rust插件
- 📝 **API兼容**: 完全兼容Halo REST API

## 技术栈

- **Web框架**: Axum
- **ORM**: Sea-ORM
- **数据库**: MySQL, PostgreSQL, Redis, MongoDB
- **全文搜索**: Tantivy
- **模板引擎**: Askama + Tera
- **WebSocket**: tokio-tungstenite

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

## 项目结构

```
flow/
├── flow/              # 主应用模块
├── flow-api/          # API定义模块
├── flow-domain/       # 领域模型模块
├── flow-infra/        # 基础设施模块
├── flow-service/      # 服务层模块
├── flow-web/          # Web层模块
├── flow-plugin/       # 插件系统模块
└── flow-migration/    # 数据库迁移模块
```

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
```

## 许可证

GPL-3.0

## 相关链接

- [Halo 原项目](https://github.com/halo-dev/halo)
- [文档](https://docs.halo.run)

