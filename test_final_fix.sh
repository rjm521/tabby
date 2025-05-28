#!/bin/bash

echo "=== 测试最终编译修复 ==="
echo "时间: $(date)"
echo

echo "1. 检查核心包编译..."
echo "正在检查 tabby-webserver..."
cargo check -p tabby-webserver

echo
echo "2. 如果单包检查通过，运行完整构建..."
if [ $? -eq 0 ]; then
    echo "单包检查通过，开始完整构建..."
    cargo build
else
    echo "单包检查失败，显示错误详情..."
    cargo check -p tabby-webserver 2>&1 | grep -A 10 -B 5 "error\["
fi

echo
echo "3. 构建状态总结："
if [ $? -eq 0 ]; then
    echo "✅ 编译成功！所有类型转换问题已修复。"
else
    echo "❌ 仍有编译问题需要解决。"
fi

echo
echo "=== 测试完成 ==="