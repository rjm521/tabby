#!/bin/bash

echo "=== 最终编译测试 ==="
echo "测试时间: $(date)"
echo

# 设置工作目录
cd /data/workspace/tabby

echo "1. 测试 tabby-db 编译..."
if cargo check -p tabby-db --quiet 2>/dev/null; then
    echo "✅ tabby-db 编译成功"
else
    echo "❌ tabby-db 编译失败"
    cargo check -p tabby-db 2>&1 | head -10
fi

echo
echo "2. 测试 tabby-schema 编译..."
if cargo check -p tabby-schema --quiet 2>/dev/null; then
    echo "✅ tabby-schema 编译成功"
else
    echo "❌ tabby-schema 编译失败"
    cargo check -p tabby-schema 2>&1 | head -10
fi

echo
echo "3. 测试 tabby-webserver 编译..."
if cargo check -p tabby-webserver --quiet 2>/dev/null; then
    echo "✅ tabby-webserver 编译成功"
else
    echo "❌ tabby-webserver 编译失败"
    cargo check -p tabby-webserver 2>&1 | head -10
fi

echo
echo "=== 编译测试完成 ==="