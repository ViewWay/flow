#!/bin/bash

# 前端代码迁移脚本
# 将 Halo 项目的 ui 目录复制到 Flow 项目的 frontend 目录

set -e

# 获取脚本所在目录的父目录（Flow 项目根目录）
FLOW_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HALO_ROOT="$(cd "$FLOW_ROOT/.." && pwd)"

echo "Flow 项目根目录: $FLOW_ROOT"
echo "Halo 项目根目录: $HALO_ROOT"

# 检查 Halo ui 目录是否存在
if [ ! -d "$HALO_ROOT/ui" ]; then
    echo "错误: 找不到 $HALO_ROOT/ui 目录"
    echo "请确保在 Halo 项目根目录下运行此脚本"
    exit 1
fi

# 检查 Flow frontend 目录是否存在
if [ ! -d "$FLOW_ROOT/frontend" ]; then
    echo "错误: 找不到 $FLOW_ROOT/frontend 目录"
    exit 1
fi

echo ""
echo "开始迁移前端代码..."
echo "源目录: $HALO_ROOT/ui"
echo "目标目录: $FLOW_ROOT/frontend"
echo ""

# 使用 rsync 复制文件，排除不必要的目录
rsync -av --progress \
    --exclude 'node_modules' \
    --exclude 'build' \
    --exclude 'dist' \
    --exclude '.git' \
    --exclude '.husky' \
    --exclude '*.log' \
    --exclude '.vite' \
    "$HALO_ROOT/ui/" "$FLOW_ROOT/frontend/"

echo ""
echo "迁移完成！"
echo ""
echo "下一步："
echo "1. cd $FLOW_ROOT/frontend"
echo "2. pnpm install"
echo "3. pnpm build:packages"
echo "4. pnpm dev"
echo ""

