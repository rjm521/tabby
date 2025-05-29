#!/bin/bash

echo "=== 测试模型配置API ==="
echo "$(date): 开始测试"

# 服务地址
BASE_URL="http://localhost:8080"

echo ""
echo "1. 检查服务是否运行..."
if curl -s --connect-timeout 5 "$BASE_URL/swagger-ui" > /dev/null; then
    echo "✅ 服务正在运行"
else
    echo "❌ 服务未运行，请先启动服务"
    echo "启动命令: ./start_chat_service.sh"
    exit 1
fi

echo ""
echo "2. 测试Swagger UI访问..."
if curl -s "$BASE_URL/swagger-ui" | grep -q "swagger"; then
    echo "✅ Swagger UI 可以访问"
    echo "   访问地址: $BASE_URL/swagger-ui"
else
    echo "⚠️  Swagger UI 访问可能有问题"
fi

echo ""
echo "3. 测试OpenAPI文档..."
if curl -s "$BASE_URL/api-docs/openapi.json" | grep -q "model_configuration"; then
    echo "✅ OpenAPI文档包含模型配置API"
else
    echo "❌ OpenAPI文档不包含模型配置API"
fi

echo ""
echo "4. 测试公开API端点（无需认证）..."
echo "   测试: GET /v1/models"
response=$(curl -s -w "HTTP_STATUS:%{http_code}" "$BASE_URL/v1/models")
http_status=$(echo "$response" | grep -o "HTTP_STATUS:[0-9]*" | cut -d: -f2)
response_body=$(echo "$response" | sed 's/HTTP_STATUS:[0-9]*$//')

if [ "$http_status" = "200" ]; then
    echo "✅ GET /v1/models 返回成功 (200)"
    echo "   响应内容预览:"
    echo "$response_body" | head -3
elif [ "$http_status" = "401" ]; then
    echo "ℹ️  GET /v1/models 需要认证 (401) - 这是正常的"
else
    echo "⚠️  GET /v1/models 返回状态码: $http_status"
fi

echo ""
echo "5. 测试需要认证的API端点..."
echo "   测试: GET /v1/user/model-preference (无token)"
response=$(curl -s -w "HTTP_STATUS:%{http_code}" "$BASE_URL/v1/user/model-preference")
http_status=$(echo "$response" | grep -o "HTTP_STATUS:[0-9]*" | cut -d: -f2)

if [ "$http_status" = "401" ]; then
    echo "✅ GET /v1/user/model-preference 正确要求认证 (401)"
else
    echo "⚠️  GET /v1/user/model-preference 返回状态码: $http_status"
fi

echo ""
echo "6. API端点清单..."
echo "   可用的模型配置API端点:"
echo "   - GET    /v1/models                     (列出可用模型)"
echo "   - POST   /v1/models                     (创建新模型)"
echo "   - GET    /v1/models/{id}                (获取特定模型)"
echo "   - PUT    /v1/models/{id}                (更新模型)"
echo "   - DELETE /v1/models/{id}                (删除模型)"
echo "   - GET    /v1/user/model-preference      (获取用户偏好)"
echo "   - PUT    /v1/user/model-preference      (更新用户偏好)"

echo ""
echo "7. 获取认证token的方法..."
echo "   如需测试需要认证的API，请使用以下方法获取token:"
echo "   1) 通过Web UI登录后获取token"
echo "   2) 使用用户注册API: POST /v1/auth/register"
echo "   3) 使用用户认证API: POST /v1/auth/token"

echo ""
echo "8. 检查数据库迁移..."
if [ -f "tabby.sqlite" ]; then
    echo "✅ 数据库文件存在"
    # 尝试检查表是否存在（如果有sqlite3命令）
    if command -v sqlite3 >/dev/null 2>&1; then
        if sqlite3 tabby.sqlite ".tables" | grep -q "available_models"; then
            echo "✅ available_models 表已创建"
        else
            echo "❌ available_models 表不存在，需要运行数据库迁移"
        fi
        if sqlite3 tabby.sqlite ".tables" | grep -q "user_model_preferences"; then
            echo "✅ user_model_preferences 表已创建"
        else
            echo "❌ user_model_preferences 表不存在，需要运行数据库迁移"
        fi
    else
        echo "ℹ️  无法检查数据库表（sqlite3命令不可用）"
    fi
else
    echo "❌ 数据库文件不存在"
fi

echo ""
echo "=== 测试完成 ==="
echo ""
echo "总结:"
echo "- 如果API能正常返回401（需要认证），说明路由配置正确"
echo "- 如果Swagger UI能访问，说明文档生成正确"
echo "- 如果数据库表存在，说明迁移已运行"
echo "- 下一步：通过Web UI或API获取认证token进行完整测试"