#!/bin/bash

echo "=== 测试所有权问题修复 ==="
echo "时间: $(date)"
echo

echo "1. 检查 tabby crate 编译..."
cargo check -p tabby

echo
echo "2. 检查 tabby-webserver 编译..."
cargo check -p tabby-webserver

echo
echo "3. 尝试完整构建..."
if cargo build; then
    echo "✅ 所有编译问题已修复！"
    exit 0
else
    echo "❌ 仍有编译问题..."
    echo "详细错误信息："
    cargo build 2>&1 | grep -A 5 -B 5 "error\["
    exit 1
fi

echo
echo "=== 测试完成 ==="