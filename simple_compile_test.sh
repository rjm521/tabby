#!/bin/bash

echo "=== 简单的编译和API测试 ==="
echo "$(date): 开始测试"

echo "1. 检查编译状态..."
timeout 60s cargo check --package tabby-webserver --no-default-features --features ee 2>&1 | head -50

echo ""
echo "2. 检查模型配置路由是否正确..."
if grep -q "model_configuration::" ee/tabby-webserver/src/routes/mod.rs; then
    echo "✅ 模型配置路由已正确引用"
else
    echo "❌ 模型配置路由引用缺失"
fi

echo ""
echo "3. 检查API文档定义..."
if grep -q "model_configuration::" ee/tabby-webserver/src/lib.rs; then
    echo "✅ 模型配置API已添加到OpenAPI文档"
else
    echo "❌ 模型配置API未添加到OpenAPI文档"
fi

echo ""
echo "4. 列出新增的API端点："
echo "   - GET    /v1/user/model-preference"
echo "   - PUT    /v1/user/model-preference"
echo "   - GET    /v1/models"
echo "   - POST   /v1/models"
echo "   - GET    /v1/models/{id}"
echo "   - PUT    /v1/models/{id}"
echo "   - DELETE /v1/models/{id}"

echo ""
echo "=== 测试完成 ==="