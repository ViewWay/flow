# 前端代码迁移指南

本文档说明如何将 Halo 的前端代码迁移到 Flow 项目中。

## 迁移步骤

### 1. 复制前端代码

在 Halo 项目根目录执行：

```bash
# 复制整个 ui 目录到 Flow 项目的 frontend 目录
cp -r ui/* flow/frontend/

# 或者使用 rsync（推荐，可以排除不必要的文件）
rsync -av --exclude 'node_modules' --exclude 'build' --exclude 'dist' \
  --exclude '.git' --exclude '.husky' \
  ui/ flow/frontend/
```

### 2. 更新配置文件

前端代码已经配置了 Vite 代理，会自动连接到 Flow 后端。无需额外配置。

### 3. 安装依赖

```bash
cd flow/frontend
pnpm install
```

### 4. 构建包

```bash
pnpm build:packages
```

### 5. 验证

启动 Flow 后端和前端，验证连接是否正常：

```bash
# 终端1：启动 Flow 后端
cd flow
cargo run

# 终端2：启动前端
cd flow/frontend
pnpm dev
```

访问 `http://localhost:3000/console/` 应该能看到前端界面。

## 文件结构说明

迁移后的目录结构：

```
flow/
├── frontend/              # 前端代码（新增）
│   ├── console-src/      # Console 前端源码
│   ├── uc-src/           # UC 前端源码
│   ├── src/              # 共享源码
│   ├── packages/         # 前端包
│   ├── package.json
│   ├── vite.config.ts
│   └── vite.uc.config.ts
├── flow/                 # Flow 后端主应用
├── flow-api/             # API 定义
├── flow-domain/          # 领域模型
├── flow-infra/           # 基础设施
├── flow-service/         # 服务层
├── flow-web/             # Web 层
└── docs/                 # 文档
```

## 注意事项

1. **不要复制 node_modules**：使用 `rsync` 时排除 `node_modules`，然后在 Flow 项目中重新安装
2. **保留 .gitignore**：确保 `.gitignore` 文件正确配置
3. **检查路径引用**：确保所有路径引用正确（相对路径应该没问题）
4. **API 客户端**：前端使用 `@halo-dev/api-client`，这个包会自动生成，无需手动修改

## 后续工作

迁移完成后，可能需要：

1. **添加静态文件服务**：在 Flow 后端添加路由，提供构建后的前端文件
2. **完善 API 适配**：根据前端实际使用情况，添加更多 API 适配
3. **配置生产环境**：配置生产环境的构建和部署流程

## 问题排查

如果迁移后出现问题：

1. 检查 `package.json` 中的依赖是否正确
2. 检查 Vite 配置文件中的路径是否正确
3. 检查 API 客户端是否正确生成
4. 查看浏览器控制台的错误信息

