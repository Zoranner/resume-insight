#!/bin/bash

# Resume Insight Docker 镜像构建脚本

set -e  # 遇到错误立即退出

# 配置变量
IMAGE_NAME="resume-insight"
IMAGE_TAG="latest"

echo "=========================================="
echo "Resume Insight Docker 镜像构建"
echo "=========================================="

# 构建 Docker 镜像
echo ""
echo "📦 正在构建 Docker 镜像..."
echo "镜像名称: ${IMAGE_NAME}:${IMAGE_TAG}"
docker build -t ${IMAGE_NAME}:${IMAGE_TAG} .

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Docker 镜像构建成功！"
    echo "=========================================="
    echo "镜像信息:"
    echo "  名称: ${IMAGE_NAME}:${IMAGE_TAG}"
    echo ""
    echo "查看镜像:"
    docker images ${IMAGE_NAME}
    echo "=========================================="
else
    echo "❌ Docker 镜像构建失败"
    exit 1
fi

echo ""
echo "✨ 完成！"
