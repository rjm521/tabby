#!/bin/bash

echo "=== 测试Swagger API文档修复 ==="
echo "$(date): 开始测试"

echo "1. 检查编译状态..."
if cargo check --bin tabby --message-format short 2>&1 | grep -q "error"; then
    echo "❌ 编译失败，查看错误："
    cargo check --bin tabby --message-format short 2>&1 | grep "error" | head -10
    exit 1
else
    echo "✅ 编译成功"
fi

echo "2. 构建项目..."
if cargo build --bin tabby --message-format short 2>&1 | grep -q "error"; then
    echo "❌ 构建失败"
    exit 1
else
    echo "✅ 构建成功"
fi

echo "3. 启动服务器（测试模式）..."
timeout 15s cargo run --bin tabby serve --host 127.0.0.1 --port 8889 > swagger_test.log 2>&1 &
SERVER_PID=$!

sleep 10

if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✅ 服务器启动成功"

    echo "4. 测试Swagger API文档..."
    if curl -s http://127.0.0.1:8889/api-docs/openapi.json | grep -q "ee_completions\|ee_chat"; then
        echo "✅ EE API端点已添加到Swagger文档"
    else
        echo "⚠️  EE API端点未在Swagger文档中找到"
    fi

    if curl -s http://127.0.0.1:8889/swagger-ui > /dev/null 2>&1; then
        echo "✅ Swagger UI可访问"
    else
        echo "❌ Swagger UI不可访问"
    fi

    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
    echo "   服务器已停止"
else
    echo "❌ 服务器启动失败"
    echo "--- 启动日志 ---"
    cat swagger_test.log 2>/dev/null || echo "无日志文件"
fi

echo "=== 测试完成 ==="