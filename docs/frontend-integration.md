# 前端适配指南

本文档说明如何将 Halo 前端连接到 Flow（Rust）后端。

## 概述

Flow 后端已经实现了与 Halo 前端兼容的 API 接口。前端代码已移植到 Flow 项目中，位于 `frontend/` 目录。

## 配置步骤

### 1. 迁移前端代码

如果还没有迁移前端代码，请参考 `frontend/MIGRATION.md` 进行迁移。

### 2. 启动 Flow 后端

确保 Flow 后端运行在 `http://localhost:8090`：

```bash
cd flow
cargo run
```

### 3. 配置前端代理

前端代理配置已经在 `frontend/src/vite/config-builder.ts` 中完成，会自动将以下路径的请求转发到 Flow 后端：

- `/api/*` - Console API 和 UC API
- `/apis/*` - Extension API
- `/actuator/*` - Actuator 端点（用于获取全局信息）
- `/themes/*` - 主题静态资源

### 4. 启动前端开发服务器

```bash
cd flow/frontend
pnpm install
pnpm build:packages
pnpm dev
```

前端将在 `http://localhost:3000` 启动，访问 `http://localhost:3000/console/` 即可使用。

## API 兼容性

### 已适配的 API

1. **获取当前用户详情**
   - 前端路径：`GET /apis/api.console.halo.run/v1alpha1/users/-`
   - Flow 后端：已实现适配，返回 `DetailedUser` 格式（包含 `user` 和 `roles` 字段）

2. **登录**
   - 前端路径：`POST /api/v1alpha1/login`
   - Flow 后端：已实现，返回 `LoginResponse`（包含 `access_token`、`token_type`、`expires_in`、`user`）

3. **登出**
   - 前端路径：`POST /api/v1alpha1/logout`
   - Flow 后端：已实现

### 需要进一步适配的 API

以下 API 可能需要根据实际使用情况进行适配：

1. **全局信息** (`/actuator/globalinfo`)
   - Flow 后端需要实现此端点，返回站点配置信息

2. **其他 Extension API**
   - 根据前端实际使用的 API，可能需要添加更多适配

## 开发注意事项

1. **CORS 配置**：Flow 后端需要配置 CORS，允许来自 `http://localhost:3000` 的请求

2. **认证方式**：前端使用 Bearer Token 或 Session Cookie 进行认证，Flow 后端已支持这两种方式

3. **API 响应格式**：确保 Flow 后端的响应格式与 Halo 原项目一致，特别是 Extension API 的响应格式

## 故障排查

### 前端无法连接到后端

1. 检查 Flow 后端是否运行在 `http://localhost:8090`
2. 检查 Vite 代理配置是否正确
3. 查看浏览器控制台的网络请求，确认请求是否被正确代理

### API 响应格式不匹配

1. 检查 Flow 后端的响应格式是否与前端期望的一致
2. 查看浏览器控制台的错误信息
3. 对比 Halo 原项目的 API 响应格式

### 认证失败

1. 检查 Flow 后端的认证中间件配置
2. 确认 JWT token 或 Session cookie 是否正确设置
3. 查看 Flow 后端的日志输出

## 下一步

1. 实现 `/actuator/globalinfo` 端点
2. 根据前端实际使用情况，添加更多 API 适配
3. 完善错误处理和响应格式
4. 添加集成测试

