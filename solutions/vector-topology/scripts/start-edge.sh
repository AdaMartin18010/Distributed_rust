#!/bin/bash

# Vector Edge 启动脚本
# 用于启动边缘节点的 Vector 服务

set -e

# 配置变量
VECTOR_BINARY="/usr/local/bin/vector"
CONFIG_FILE="/etc/vector/edge.toml"
LOG_DIR="/var/log/vector"
DATA_DIR="/var/lib/vector"

# 创建必要的目录
mkdir -p "$LOG_DIR" "$DATA_DIR"

# 设置环境变量
export NODE_ID="${NODE_ID:-$(hostname)}"
export RUST_LOG="${RUST_LOG:-info}"

# 验证配置文件
if [ ! -f "$CONFIG_FILE" ]; then
    echo "错误: 配置文件不存在: $CONFIG_FILE"
    exit 1
fi

# 验证 Vector 二进制文件
if [ ! -f "$VECTOR_BINARY" ]; then
    echo "错误: Vector 二进制文件不存在: $VECTOR_BINARY"
    exit 1
fi

# 验证配置文件语法
echo "验证配置文件..."
"$VECTOR_BINARY" validate "$CONFIG_FILE"

if [ $? -ne 0 ]; then
    echo "错误: 配置文件验证失败"
    exit 1
fi

echo "配置文件验证成功"

# 启动 Vector
echo "启动 Vector Edge 服务..."
echo "节点 ID: $NODE_ID"
echo "配置文件: $CONFIG_FILE"
echo "日志目录: $LOG_DIR"

exec "$VECTOR_BINARY" --config "$CONFIG_FILE" \
    --log-level "$RUST_LOG" \
    --data-dir "$DATA_DIR" \
    --log-dir "$LOG_DIR"
