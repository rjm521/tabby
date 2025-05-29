#!/bin/bash

echo "=== 测试用户模型配置API的Swagger文档 ==="
echo "$(date): 开始测试"

echo "1. 尝试启动服务器..."
timeout 20s cargo run --bin tabby serve --host 127.0.0.1 --port 8888 > swagger_api_test.log 2>&1 &
SERVER_PID=$!

sleep 15

if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✅ 服务器启动成功"

    echo "2. 测试Swagger API文档..."

    # 检查OpenAPI JSON
    if curl -s http://127.0.0.1:8888/api-docs/openapi.json > api_docs.json 2>/dev/null; then
        echo "✅ OpenAPI JSON可访问"

        # 检查用户模型配置相关的API端点
        if grep -q "get_user_model_preference\|update_user_model_preference\|list_available_models" api_docs.json; then
            echo "✅ 用户模型配置API已添加到Swagger文档"

            # 统计API端点数量
            model_endpoints=$(grep -o '"operationId".*model' api_docs.json | wc -l)
            echo "   模型配置相关端点数量: $model_endpoints"

            # 列出具体的端点
            echo "   检测到的端点："
            grep -o '"operationId":"[^"]*model[^"]*"' api_docs.json | sed 's/"operationId":"//; s/"$//' | sed 's/^/     - /'
        else
            echo "❌ 用户模型配置API未在Swagger文档中找到"
        fi

        # 检查schema定义
        if grep -q "UserModelPreferenceResponse\|AvailableModelResponse" api_docs.json; then
            echo "✅ 模型配置相关的Schema已添加"
        else
            echo "❌ 模型配置相关的Schema未找到"
        fi
    else
        echo "❌ 无法访问OpenAPI JSON"
    fi

    # 检查Swagger UI
    if curl -s http://127.0.0.1:8888/swagger-ui > /dev/null 2>&1; then
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
    tail -20 swagger_api_test.log 2>/dev/null || echo "无日志文件"
fi

echo ""
echo "=== 新增的用户模型配置API端点 ==="
echo "1. 用户模型偏好管理:"
echo "   GET    /v1/user/model-preference      - 获取用户模型偏好"
echo "   PUT    /v1/user/model-preference      - 更新用户模型偏好"
echo ""
echo "2. 可用模型管理:"
echo "   GET    /v1/models                     - 列出可用模型"
echo "   POST   /v1/models                     - 创建新模型"
echo "   GET    /v1/models/{id}                - 获取特定模型"
echo "   PUT    /v1/models/{id}                - 更新模型信息"
echo "   DELETE /v1/models/{id}                - 删除模型"
echo ""
echo "=== 测试完成 ==="