#!/bin/bash

echo "=== Tabby用户模型配置API完整诊断 ==="
echo "$(date): 开始全面诊断"

# 检查基本环境
echo ""
echo "1. 环境检查..."
echo "   工作目录: $(pwd)"
echo "   Rust版本: $(rustc --version 2>/dev/null || echo '未安装')"
echo "   Cargo版本: $(cargo --version 2>/dev/null || echo '未安装')"

# 检查关键文件是否存在
echo ""
echo "2. 关键文件检查..."
files=(
    "ee/tabby-webserver/src/routes/model_configuration.rs"
    "ee/tabby-webserver/src/routes/mod.rs"
    "ee/tabby-webserver/src/lib.rs"
    "ee/tabby-webserver/src/service/model_configuration.rs"
    "ee/tabby-db/migrations/0049_add_available_models_table.up.sql"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✅ $file"
    else
        echo "   ❌ $file (缺失)"
    fi
done

# 检查模块引用
echo ""
echo "3. 模块引用检查..."
if grep -q "pub mod model_configuration;" ee/tabby-webserver/src/routes/mod.rs; then
    echo "   ✅ model_configuration模块已在routes/mod.rs中声明"
else
    echo "   ❌ model_configuration模块未在routes/mod.rs中声明"
fi

if grep -q "model_configuration::" ee/tabby-webserver/src/lib.rs; then
    echo "   ✅ model_configuration API已添加到OpenAPI文档"
else
    echo "   ❌ model_configuration API未添加到OpenAPI文档"
fi

# 检查路由配置
echo ""
echo "4. 路由配置检查..."
if grep -q "/v1/user/model-preference" ee/tabby-webserver/src/routes/mod.rs; then
    echo "   ✅ 用户模型偏好路由已配置"
else
    echo "   ❌ 用户模型偏好路由未配置"
fi

if grep -q "/v1/models" ee/tabby-webserver/src/routes/mod.rs; then
    echo "   ✅ 模型管理路由已配置"
else
    echo "   ❌ 模型管理路由未配置"
fi

# 检查服务集成
echo ""
echo "5. 服务集成检查..."
if grep -q "model_configuration::create" ee/tabby-webserver/src/webserver.rs; then
    echo "   ✅ 模型配置服务已在webserver中集成"
else
    echo "   ❌ 模型配置服务未在webserver中集成"
fi

# 尝试检查编译状态（快速检查）
echo ""
echo "6. 编译状态检查..."
if timeout 30s cargo check --package tabby-webserver --no-default-features --features ee --message-format short 2>&1 | grep -q "Finished"; then
    echo "   ✅ 编译检查通过"
elif timeout 30s cargo check --package tabby-webserver --no-default-features --features ee --message-format short 2>&1 | grep -q "error"; then
    echo "   ❌ 编译有错误"
    echo "   最近的编译错误："
    timeout 30s cargo check --package tabby-webserver --no-default-features --features ee --message-format short 2>&1 | grep "error" | head -5
else
    echo "   ⚠️  编译状态未知（可能正在编译或超时）"
fi

# 检查数据库相关文件
echo ""
echo "7. 数据库检查..."
if [ -f "ee/tabby-db/migrations/0049_add_available_models_table.up.sql" ]; then
    echo "   ✅ 数据库迁移文件存在"
    echo "   迁移文件大小: $(wc -l < ee/tabby-db/migrations/0049_add_available_models_table.up.sql) 行"
else
    echo "   ❌ 数据库迁移文件缺失"
fi

# 列出所有新增的API端点
echo ""
echo "8. API端点清单..."
echo "   用户模型偏好管理:"
echo "     GET    /v1/user/model-preference      - 获取用户模型偏好"
echo "     PUT    /v1/user/model-preference      - 更新用户模型偏好"
echo ""
echo "   可用模型管理:"
echo "     GET    /v1/models                     - 列出可用模型"
echo "     POST   /v1/models                     - 创建新模型"
echo "     GET    /v1/models/{id}                - 获取特定模型"
echo "     PUT    /v1/models/{id}                - 更新模型信息"
echo "     DELETE /v1/models/{id}                - 删除模型"

# 检查端口是否被占用
echo ""
echo "9. 端口检查..."
if netstat -tlnp 2>/dev/null | grep -q ":8080"; then
    echo "   ⚠️  端口8080已被占用"
else
    echo "   ✅ 端口8080可用"
fi

# 建议的修复步骤
echo ""
echo "10. 建议的修复步骤..."
echo "   如果API无法访问，请尝试以下步骤："
echo "   1) 确保所有文件都已正确创建和修改"
echo "   2) 运行数据库迁移: cargo run --bin tabby --features migrate migrate"
echo "   3) 重新编译项目: cargo build --release"
echo "   4) 启动服务: cargo run --bin tabby serve --host 127.0.0.1 --port 8080"
echo "   5) 访问Swagger UI: http://localhost:8080/swagger-ui"

# 测试建议
echo ""
echo "11. 测试建议..."
echo "   启动服务后，可以使用以下curl命令测试:"
echo "   # 列出可用模型"
echo "   curl -X GET http://localhost:8080/v1/models"
echo ""
echo "   # 获取用户偏好（需要认证）"
echo "   curl -X GET http://localhost:8080/v1/user/model-preference \\"
echo "     -H \"Authorization: Bearer YOUR_TOKEN\""

echo ""
echo "=== 诊断完成 ==="