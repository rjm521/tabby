#!/bin/bash

echo "=== 重新启动Tabby服务以应用模型配置API修复 ==="

# 停止现有服务
echo "1. 停止现有服务..."
if [ -f "chat_service.pid" ]; then
    PID=$(cat chat_service.pid)
    if ps -p "$PID" > /dev/null 2>&1; then
        echo "   停止服务 (PID: $PID)..."
        kill "$PID"
        sleep 3
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "   强制停止服务..."
            kill -9 "$PID"
        fi
    fi
    rm -f chat_service.pid
    echo "   ✅ 服务已停止"
else
    echo "   ℹ️  没有找到运行中的服务"
fi

# 清理旧的编译缓存（如果需要）
echo ""
echo "2. 清理编译缓存（可选）..."
if [ -d "target" ]; then
    echo "   清理target目录..."
    # 只清理debug目录，保留依赖缓存
    rm -rf target/debug/tabby* 2>/dev/null || true
    echo "   ✅ 编译缓存已清理"
fi

# 运行数据库迁移（确保表存在）
echo ""
echo "3. 运行数据库迁移..."
if timeout 60s cargo run --features ee --bin tabby -- migrate 2>/dev/null; then
    echo "   ✅ 数据库迁移完成"
else
    echo "   ⚠️  数据库迁移可能失败，但继续启动服务"
fi

# 启动服务
echo ""
echo "4. 启动服务（启用EE功能）..."
echo "   启动命令: cargo run --features ee --bin tabby -- serve --device cpu --port 8080"

# 使用修复后的启动脚本
if [ -f "start_chat_service.sh" ]; then
    echo "   使用修复后的启动脚本..."
    chmod +x start_chat_service.sh
    ./start_chat_service.sh
else
    # 手动启动
    echo "   手动启动服务..."
    nohup cargo run --features ee --bin tabby -- serve --device cpu --port 8080 > logs/service_$(date +%Y%m%d_%H%M%S).log 2>&1 &
    echo $! > chat_service.pid
    echo "   ✅ 服务已启动 (PID: $(cat chat_service.pid))"
fi

echo ""
echo "5. 等待服务启动..."
sleep 10

echo ""
echo "6. 验证服务..."
if curl -s --connect-timeout 5 "http://localhost:8080/swagger-ui" > /dev/null; then
    echo "   ✅ 服务已启动，Swagger UI可访问"
    echo ""
    echo "🎉 修复完成！请访问以下地址："
    echo "   - Swagger UI: http://localhost:8080/swagger-ui"
    echo "   - OpenAPI文档: http://localhost:8080/api-docs/openapi.json"
    echo ""
    echo "📖 现在应该能看到以下模型配置API："
    echo "   - GET/PUT /v1/user/model-preference"
    echo "   - GET/POST /v1/models"
    echo "   - GET/PUT/DELETE /v1/models/{id}"
else
    echo "   ❌ 服务启动可能失败"
    echo "   请检查日志: tail -f logs/*.log"
fi

echo ""
echo "=== 重启完成 ==="