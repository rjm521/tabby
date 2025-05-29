#!/bin/bash

echo "=== 测试启动修复 ==="
echo "$(date): 开始测试"

echo "1. 检查编译状态..."
if cargo build --bin tabby 2>&1 | grep -q "error"; then
    echo "❌ 编译失败"
    cargo build --bin tabby 2>&1 | tail -20
    exit 1
else
    echo "✅ 编译成功"
fi

echo "2. 尝试启动服务 (超时10秒)..."
timeout 10s cargo run --bin tabby serve --model StarCoder-1B 2>&1 > startup_test.log &
TABBY_PID=$!

sleep 8

if ps -p $TABBY_PID > /dev/null; then
    echo "✅ 启动成功 - 服务正在运行"
    kill $TABBY_PID 2>/dev/null
    echo "服务已停止"
else
    echo "❌ 启动失败 - 检查错误日志"
    if [ -f startup_test.log ]; then
        echo "--- 启动日志 ---"
        tail -20 startup_test.log
    fi
fi

echo "=== 测试完成 ==="