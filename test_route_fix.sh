#!/bin/bash

echo "=== 测试路由冲突修复 ==="
echo "$(date): 开始测试"

echo "1. 构建项目..."
cargo build --bin tabby >/dev/null 2>&1

if [ $? -eq 0 ]; then
    echo "✅ 构建成功"
else
    echo "❌ 构建失败"
    exit 1
fi

echo "2. 启动服务器测试..."
# 启动服务器，5秒后检查
timeout 10s cargo run --bin tabby serve --host 127.0.0.1 --port 8888 > startup_test.log 2>&1 &
SERVER_PID=$!

echo "   启动进程 PID: $SERVER_PID"
sleep 7

# 检查进程是否还在运行
if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✅ 服务器启动成功！"

    # 尝试简单的健康检查
    if curl -s http://127.0.0.1:8888/v1/health >/dev/null 2>&1; then
        echo "✅ 健康检查通过"
    else
        echo "⚠️  健康检查未通过（可能需要更长启动时间）"
    fi

    # 停止服务器
    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
    echo "   服务器已停止"
else
    echo "❌ 服务器启动失败"
    echo "--- 启动日志 ---"
    cat startup_test.log
fi

echo "=== 测试完成 ==="