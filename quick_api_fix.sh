#!/bin/bash

echo "=== Tabby 模型配置API 快速修复和测试 ==="

# 检查服务状态
if [ -f "chat_service.pid" ] && ps -p "$(cat chat_service.pid)" > /dev/null 2>&1; then
    echo "✅ 服务正在运行 (PID: $(cat chat_service.pid))"
    SERVICE_RUNNING=true
else
    echo "❌ 服务未运行"
    SERVICE_RUNNING=false
fi

# 手动简单测试（如果服务在运行）
if [ "$SERVICE_RUNNING" = true ]; then
    echo ""
    echo "测试API端点..."

    # 测试Swagger文档
    echo "1. 测试Swagger UI..."
    if timeout 5s curl -s "http://localhost:8080/swagger-ui" > /dev/null 2>&1; then
        echo "   ✅ Swagger UI 可访问"
    else
        echo "   ❌ Swagger UI 无法访问"
    fi

    # 测试OpenAPI文档
    echo "2. 测试OpenAPI文档..."
    if timeout 5s curl -s "http://localhost:8080/api-docs/openapi.json" 2>/dev/null | grep -q "model_configuration" 2>/dev/null; then
        echo "   ✅ OpenAPI文档包含模型配置API"
    else
        echo "   ❌ OpenAPI文档不包含模型配置API"
    fi

    # 测试API端点
    echo "3. 测试API端点..."
    API_STATUS=$(timeout 5s curl -s -w "%{http_code}" -o /dev/null "http://localhost:8080/v1/models" 2>/dev/null || echo "000")
    if [ "$API_STATUS" = "200" ]; then
        echo "   ✅ GET /v1/models 返回 200"
    elif [ "$API_STATUS" = "401" ]; then
        echo "   ✅ GET /v1/models 返回 401 (需要认证，这是正常的)"
    elif [ "$API_STATUS" = "404" ]; then
        echo "   ❌ GET /v1/models 返回 404 (路由未找到)"
    else
        echo "   ⚠️  GET /v1/models 返回状态码: $API_STATUS"
    fi

    # 测试用户偏好API
    USER_PREF_STATUS=$(timeout 5s curl -s -w "%{http_code}" -o /dev/null "http://localhost:8080/v1/user/model-preference" 2>/dev/null || echo "000")
    if [ "$USER_PREF_STATUS" = "401" ]; then
        echo "   ✅ GET /v1/user/model-preference 返回 401 (需要认证，这是正常的)"
    elif [ "$USER_PREF_STATUS" = "404" ]; then
        echo "   ❌ GET /v1/user/model-preference 返回 404 (路由未找到)"
    else
        echo "   ⚠️  GET /v1/user/model-preference 返回状态码: $USER_PREF_STATUS"
    fi
fi

echo ""
echo "检查关键文件..."

# 检查路由文件
if grep -q "model_configuration::" ee/tabby-webserver/src/routes/mod.rs 2>/dev/null; then
    echo "✅ 路由模块正确引用"
else
    echo "❌ 路由模块引用有问题"
fi

# 检查OpenAPI文档
if grep -q "model_configuration::" ee/tabby-webserver/src/lib.rs 2>/dev/null; then
    echo "✅ OpenAPI文档包含模型配置路由"
else
    echo "❌ OpenAPI文档缺少模型配置路由"
fi

# 检查数据库
if [ -f "tabby.sqlite" ]; then
    echo "✅ 数据库文件存在"
else
    echo "❌ 数据库文件不存在"
fi

echo ""
echo "可能的问题和解决方案："
echo ""

if [ "$SERVICE_RUNNING" = false ]; then
    echo "1. 服务未运行："
    echo "   解决方案: ./start_chat_service.sh"
fi

if [ "$API_STATUS" = "404" ] || [ "$USER_PREF_STATUS" = "404" ]; then
    echo "2. API路由404错误："
    echo "   可能原因: 路由配置有问题或服务需要重启"
    echo "   解决方案: 重启服务应用新的路由配置"
    echo "   命令: ./stop_chat_service.sh && ./start_chat_service.sh"
fi

echo ""
echo "3. 如果服务重启后仍有问题，可能需要："
echo "   - 检查编译错误"
echo "   - 运行数据库迁移"
echo "   - 验证路由中间件配置"

echo ""
echo "=== 快速测试完成 ==="