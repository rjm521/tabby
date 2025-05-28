#!/bin/bash

echo "=== 测试编译修复情况 ==="
echo "时间: $(date)"
echo

echo "1. 检查核心包编译..."
cargo check -p tabby-webserver 2>&1 | head -50

echo
echo "2. 尝试完整构建..."
make dev-build 2>&1 | tail -20

echo
echo "=== 测试完成 ==="