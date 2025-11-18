# Flow 前端项目

这是 Flow 项目的前端部分，基于 Halo 的前端代码移植而来。

## 目录结构

```
frontend/
├── console-src/          # Console 前端源码
├── uc-src/               # UC (User Content) 前端源码
├── src/                  # 共享源码
├── packages/             # 前端包（api-client, components, editor, shared等）
├── package.json          # 项目配置
├── vite.config.ts        # Console Vite 配置
├── vite.uc.config.ts    # UC Vite 配置
└── README.md            # 本文档
```

## 快速开始

### 1. 从 Halo 项目复制前端代码

```bash
# 在 halo 项目根目录执行
cd /Users/yimiliya/github/halo

# 复制前端代码到 Flow 项目
cp -r ui/* flow/frontend/
```

### 2. 安装依赖

```bash
cd flow/frontend
pnpm install
```

### 3. 构建包

```bash
pnpm build:packages
```

### 4. 启动开发服务器

```bash
# 启动 Console 前端（端口 3000）
pnpm dev:console

# 或启动 UC 前端（端口 4000）
pnpm dev:uc

# 或同时启动两个（推荐）
pnpm dev
```

## 配置说明

### Vite 代理配置

前端已配置代理，会自动将 API 请求转发到 Flow 后端（`http://localhost:8090`）：

- `/api/*` → Flow 后端 Console API 和 UC API
- `/apis/*` → Flow 后端 Extension API
- `/actuator/*` → Flow 后端 Actuator 端点
- `/themes/*` → Flow 后端主题静态资源

### 开发环境

- Console 前端：`http://localhost:3000/console/`
- UC 前端：`http://localhost:4000/uc/`

### 生产构建

```bash
# 构建 Console 前端
pnpm build:console

# 构建 UC 前端
pnpm build:uc

# 构建所有前端
pnpm build
```

构建产物将输出到 `build/dist/console/` 和 `build/dist/uc/` 目录。

## 与 Flow 后端集成

### 开发模式

在开发模式下，前端通过 Vite 代理连接到 Flow 后端。确保 Flow 后端运行在 `http://localhost:8090`。

### 生产模式

在生产模式下，可以将构建后的前端文件：

1. **选项1：由 Flow 后端提供静态文件服务**
   - 将构建产物复制到 Flow 后端的静态资源目录
   - Flow 后端需要添加静态文件服务路由

2. **选项2：使用独立的 Web 服务器**
   - 使用 Nginx、Apache 等 Web 服务器提供前端文件
   - 配置反向代理将 API 请求转发到 Flow 后端

## 注意事项

1. **API 兼容性**：确保 Flow 后端实现了前端需要的所有 API 端点
2. **CORS 配置**：Flow 后端需要配置 CORS，允许前端域名
3. **认证方式**：前端使用 Bearer Token 或 Session Cookie，确保 Flow 后端支持

## 故障排查

### 前端无法连接到后端

1. 检查 Flow 后端是否运行在 `http://localhost:8090`
2. 检查 Vite 代理配置是否正确
3. 查看浏览器控制台的网络请求

### API 响应格式不匹配

1. 检查 Flow 后端的响应格式
2. 查看浏览器控制台的错误信息
3. 参考 `docs/frontend-integration.md` 了解 API 适配情况

## 相关文档

- [前端集成指南](../docs/frontend-integration.md)
- [Flow 项目 README](../README.md)

