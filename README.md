# Tabby AI 代码助手

**最后更新：** 2024年12月19日

## 📋 项目概览

Tabby 是一个自托管的AI代码完成助手，类似于GitHub Copilot，专为提供隐私保护的代码补全服务而设计。本项目包含了用户模型配置功能的完整实现。

## 🎯 用户模型配置功能

### 功能描述
允许用户在Tabby中个性化配置和管理AI代码补全模型，包括：
- 选择不同的代码补全模型
- 配置模型参数
- 保存用户偏好设置
- 提供RESTful API和GraphQL接口

### 📂 核心组件

#### 后端实现
```
ee/tabby-webserver/src/service/model_configuration.rs  # 模型配置服务
ee/tabby-webserver/src/routes/ee_completions.rs        # EE 代码补全路由
ee/tabby-webserver/src/routes/ee_chat.rs               # EE 聊天路由
crates/tabby/src/services/completion.rs                # 补全服务核心
```

#### 数据库层
- 用户模型偏好存储
- 可用模型信息管理
- 运行时查询接口

#### API层
- **RESTful API**: `/v1/completions`, `/v1/chat/completions`, `/v1/ee/*`
- **GraphQL API**: 用户模型配置查询和变更
- **认证集成**: 支持基于JWT的用户认证
- **完整Swagger文档**: 所有端点均有详细的OpenAPI文档

---

## 🛠️ 编译和启动修复历程

### ✅ 修复总结

经过6轮系统性修复，用户模型配置功能已完全可用：

1. **第1-2轮：基础编译问题** - SQLx数据库集成、模块依赖、导入路径
2. **第3轮：类型系统问题** - 类型转换、孤儿规则、字符串处理
3. **第4轮：所有权问题** - Rust内存安全、变量生命周期管理
4. **第5轮：路由冲突问题** - EE版本与基础版本路由重叠解决
5. **🆕 第6轮：Swagger文档问题** - OpenAPI注解缺失导致API文档不完整

### 🔧 关键技术修复

#### 🆕 Swagger文档完整化（第6轮重点修复）
**问题**：服务启动成功但Swagger UI中缺少EE API端点文档
```
原因：EE路由函数缺少 #[utoipa::path] 注解
```

**解决方案**：
- 为所有EE路由添加完整的OpenAPI注解
- 为数据结构添加ToSchema derive宏
- 更新EEApiDoc包含新的路由和schema
- 在独立路径上启用EE路由（`/v1/ee/*`）

**修复文件**：
- `ee/tabby-webserver/src/routes/ee_completions.rs` - 添加OpenAPI注解
- `ee/tabby-webserver/src/routes/ee_chat.rs` - 添加OpenAPI注解
- `ee/tabby-webserver/src/lib.rs` - 更新EEApiDoc定义
- `ee/tabby-webserver/src/routes/mod.rs` - 添加新路由

#### 路由冲突解决（第5轮重点修复）
**问题**：EE版本重复定义了基础版本已有的路由
```
错误：Overlapping method route. Handler for 'POST /v1/completions' already exists
```

**解决方案**：
- 注释掉EE版本中的重复路由定义
- 保持基础路由功能完整性
- 为未来EE功能扩展预留灵活性

**文件位置**：`ee/tabby-webserver/src/routes/mod.rs`

#### Rust类型系统优化
- **所有权管理**：正确处理值移动和借用
- **类型转换**：安全的String到ID转换
- **内存安全**：零拷贝数据传递优化

---

## 🚀 API端点总览

### 🔄 基础端点
```
POST   /v1/completions              # 基础代码补全服务
POST   /v1/chat/completions         # 基础聊天补全服务
GET    /v1/health                   # 健康状态检查
GET    /v1beta/models               # 可用模型列表
POST   /v1/events                   # 事件日志记录
POST   /v1/test/generate           # 🆕 测试用例生成服务
```

### ✨ EE企业端点（新增）
```
POST   /v1/ee/completions           # 🆕 EE代码补全（支持用户模型偏好）
POST   /v1/ee/chat/completions      # 🆕 EE聊天补全（支持用户模型偏好）
GET    /v1beta/server_setting       # EE服务器设置
POST   /v1/graphql                  # GraphQL查询接口
POST   /v1/auth/token               # 用户认证令牌
POST   /v1/auth/register            # 用户注册
```

### 📚 API文档访问
- **Swagger UI**: `http://localhost:8080/swagger-ui`
- **OpenAPI JSON**: `http://localhost:8080/api-docs/openapi.json`
- **GraphiQL**: `http://localhost:8080/graphiql`

---

## 🚀 使用指南

### 编译项目
```bash
# 构建整个项目
cargo build

# 构建Tabby服务器
cargo build --bin tabby
```

### 启动服务
```bash
# 启动Tabby服务器（包含测试代理服务）
./start_chat_service.sh -p 8080

# 或者使用cargo直接启动
cargo run --bin tabby serve --host 0.0.0.0 --port 8080
```

### API使用示例

#### 🆕 测试用例生成API
```bash
# 生成接口测试用例
curl -X POST http://localhost:8080/v1/test/generate \
  -H "Content-Type: application/json" \
  -d '{
    "api_desc": "POST /v1/chat/completions，用于生成聊天补全。参数：messages（消息列表），返回：生成的回复文本"
  }'

# 返回示例
{
  "test_case": "1. 测试场景概述\n\
    - 验证聊天补全接口的基本功能\n\
    - 验证异常输入的处理\n\
    - 验证边界条件的处理\n\
    \n\
    2. 测试用例列表\n\
    \n\
    用例1：正常聊天补全\n\
    - 前置条件：服务正常运行\n\
    - 测试步骤：\n\
      1. 发送包含有效消息的请求\n\
      2. 验证返回状态码为200\n\
      3. 验证返回的文本符合预期\n\
    - 预期结果：成功生成补全文本\n\
    \n\
    用例2：空消息列表\n\
    - 前置条件：服务正常运行\n\
    - 测试步骤：\n\
      1. 发送空消息列表的请求\n\
      2. 验证返回状态码为400\n\
    - 预期结果：返回适当的错误信息\n\
    \n\
    3. 测试数据准备\n\
    - 准备有效的消息列表\n\
    - 准备各种异常输入数据\n\
    \n\
    4. 注意事项\n\
    - 确保测试环境稳定\n\
    - 注意请求频率限制\n\
    - 关注响应时间"
}
```

#### 🆕 EE代码补全API（支持用户偏好）
```bash
curl -X POST http://localhost:8080/v1/ee/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "prompt": "fn main() {",
    "model": "StarCoder-1B"
  }'
```

#### 🆕 EE聊天API（支持用户偏好）
```bash
curl -X POST http://localhost:8080/v1/ee/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "messages": [
      {"role": "user", "content": "如何在Rust中创建向量？"}
    ],
    "model": "CodeLlama-7B"
  }'
```

#### 基础代码补全API
```bash
curl -X POST http://localhost:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "language": "rust",
    "segments": {
      "prefix": "fn main() {"
    }
  }'
```

#### GraphQL查询
```bash
curl -X POST http://localhost:8080/v1/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "query": "query { me { email } }"
  }'
```

---

## 📚 开发文档

### 项目架构
```
tabby/
├── crates/
│   ├── tabby/                      # 核心Tabby服务
│   ├── tabby-inference/            # AI推理引擎
│   └── tabby-common/               # 共享组件
├── ee/
│   ├── tabby-webserver/            # EE Web服务器
│   ├── tabby-db/                   # 数据库层
│   └── tabby-schema/               # GraphQL Schema
└── clients/                        # 客户端集成
    ├── vscode/                     # VS Code扩展
    ├── intellij/                   # IntelliJ插件
    └── vim/                        # Vim插件
```

### 数据库Schema
- `user_model_preferences`: 用户模型偏好设置
- `available_models`: 可用AI模型信息
- 运行时支持SQLite和PostgreSQL

### 技术栈
- **后端**: Rust, Axum, SQLx, Juniper (GraphQL)
- **数据库**: SQLite/PostgreSQL
- **AI引擎**: 支持多种开源LLM模型
- **认证**: JWT令牌
- **API文档**: OpenAPI/Swagger, utoipa
- **客户端**: TypeScript, VS Code API

---

## 🧪 测试

### 单元测试
```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test -p tabby-webserver
```

### API测试
```bash
# 启动测试服务器
cargo run --bin tabby serve --port 9090

# 测试Swagger文档
curl http://localhost:9090/api-docs/openapi.json | jq '.paths'

# 测试EE端点
curl -X POST http://localhost:9090/v1/ee/completions \
  -H "Content-Type: application/json" \
  -d '{"prompt": "test"}'
```

---

## 🛡️ 代码质量保证

### 编译检查
✅ 所有编译错误已修复
✅ 类型安全得到保证
✅ 内存安全验证通过
✅ 所有警告已处理
✅ **Swagger文档完整性验证**

### 性能优化
- 零拷贝数据传递
- 异步I/O处理
- 连接池复用
- 智能缓存策略

---

## 📄 相关文档

- [`COMPILATION_FIX_STATUS.md`](./COMPILATION_FIX_STATUS.md) - 详细修复状态
- [`COMPILATION_FIX_SUMMARY.md`](./COMPILATION_FIX_SUMMARY.md) - 修复总结
- [`database_schema.md`](./database_schema.md) - 数据库设计文档

---

## 🤝 贡献指南

1. Fork项目仓库
2. 创建功能分支：`git checkout -b feature/your-feature`
3. 提交变更：`git commit -am 'Add some feature'`
4. 推送分支：`git push origin feature/your-feature`
5. 创建Pull Request

---

## 📞 支持与反馈

如有问题或建议，请：
- 创建GitHub Issue
- 查看项目Wiki
- 参考修复文档获取故障排除帮助
- 访问Swagger UI查看完整API文档

**项目状态**：✅ 用户模型配置功能开发完成，包括完整的Swagger API文档！ 🎉
**新增功能**：🆕 tRPC测试生成工具已开发完成，支持AI驱动的Go测试用例自动生成！

## 🚀 功能特性

- **智能代码补全**：基于上下文的代码自动补全
- **AI聊天助手**：与AI进行编程相关的对话
- **用户模型偏好**：用户可以设置首选的AI模型
- **企业级功能**：支持用户认证和个性化配置
- **完整API文档**：提供Swagger UI和OpenAPI规范
- **🆕 tRPC测试生成工具**：AI驱动的tRPC-Go项目测试用例自动生成

## 📚 API端点

### 基础端点
- `POST /v1/completions` - 代码补全
- `POST /v1/chat/completions` - 聊天补全
- `GET /v1/health` - 健康检查
- `GET /v1beta/models` - 模型列表

### 企业版端点
- `POST /v1/ee/completions` - 企业版代码补全（支持用户偏好）
- `POST /v1/ee/chat/completions` - 企业版聊天补全（支持用户偏好）

### 🆕 用户模型配置API

#### 用户模型偏好管理
- `GET /v1/user/model-preference` - 获取用户模型偏好
- `PUT /v1/user/model-preference` - 更新用户模型偏好

#### 可用模型管理
- `GET /v1/models` - 列出可用模型
- `POST /v1/models` - 创建新模型（管理员）
- `GET /v1/models/{id}` - 获取特定模型
- `PUT /v1/models/{id}` - 更新模型信息（管理员）
- `DELETE /v1/models/{id}` - 删除模型（管理员）

## 🔧 使用方法

### 1. 启动服务
```bash
cargo run --bin tabby serve --host 127.0.0.1 --port 8080
```

### 2. 访问API文档
- **Swagger UI**: http://localhost:8080/swagger-ui
- **OpenAPI JSON**: http://localhost:8080/api-docs/openapi.json

### 3. 用户模型偏好设置

#### 获取用户偏好
```bash
curl -X GET http://localhost:8080/v1/user/model-preference \
  -H "Authorization: Bearer YOUR_TOKEN"
```

#### 更新用户偏好
```bash
curl -X PUT http://localhost:8080/v1/user/model-preference \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "completion_model_id": "StarCoder-1B",
    "chat_model_id": "CodeLlama-7B"
  }'
```

### 4. 模型管理

#### 列出可用模型
```bash
# 列出所有模型
curl -X GET http://localhost:8080/v1/models

# 按类型筛选
curl -X GET http://localhost:8080/v1/models?model_type=completion
```

#### 创建新模型（管理员）
```bash
curl -X POST http://localhost:8080/v1/models \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -d '{
    "name": "custom-model",
    "display_name": "Custom Model",
    "model_type": "completion",
    "provider": "openai",
    "performance_tier": "balanced",
    "max_tokens": 4096,
    "context_window": 8192,
    "enabled": true,
    "description": "Custom model for specific use cases"
  }'
```

## 📊 数据结构

### 用户模型偏好
```json
{
  "id": "user_pref_123",
  "user_id": "user_456",
  "completion_model_id": "StarCoder-1B",
  "chat_model_id": "CodeLlama-7B",
  "created_at": "2024-12-19T10:00:00Z",
  "updated_at": "2024-12-19T10:00:00Z"
}
```

### 可用模型
```json
{
  "id": "model_123",
  "name": "starcoder-1b",
  "display_name": "StarCoder 1B",
  "model_type": "completion",
  "provider": "huggingface",
  "performance_tier": "balanced",
  "max_tokens": 4096,
  "context_window": 8192,
  "enabled": true,
  "description": "Fast and efficient code completion model",
  "created_at": "2024-12-19T10:00:00Z",
  "updated_at": "2024-12-19T10:00:00Z"
}
```

## 🔐 认证

大部分API端点需要JWT认证。在请求头中包含：
```
Authorization: Bearer YOUR_JWT_TOKEN
```

## 🏗️ 架构

### 核心组件
- **tabby-webserver**: Web服务器和API路由
- **tabby-schema**: GraphQL schema和数据模型
- **tabby-db**: 数据库层和迁移
- **tabby-common**: 共享工具和类型
- **🆕 trpc-test-generator**: tRPC-Go项目的AI测试生成工具

### 数据库
项目使用SQLite数据库，包含以下主要表：
- `users` - 用户信息
- `user_model_preferences` - 用户模型偏好
- `available_models` - 可用AI模型配置

## 🧪 tRPC测试生成工具

### 功能概述
基于AI驱动的tRPC-Go项目测试用例自动生成工具，类似于qodo-ai/qodo-cover，专门为tRPC-Go项目设计。

### 核心特性
- 🤖 **AI驱动**: 使用rig框架集成多种LLM（OpenAI、Claude、本地Tabby）
- 🔍 **智能分析**: 自动分析tRPC路由定义和Go代码结构
- 📝 **自动生成**: 生成符合Go testing标准的高质量测试代码
- 📊 **覆盖率验证**: 验证生成的测试覆盖率并提供详细报告

### 支持的测试类型
- ✅ **单元测试**: 测试tRPC handler的核心逻辑
- ✅ **集成测试**: 测试完整的tRPC调用流程
- ✅ **边界测试**: 测试输入参数的边界情况
- ✅ **错误处理测试**: 测试各种错误情况的处理
- ✅ **性能测试**: 生成基准测试代码
- ✅ **Mock测试**: 生成Mock对象和测试

### 快速使用
```bash
# 构建工具
cd crates/trpc-test-generator
cargo build --release

# 分析tRPC项目
./target/release/trpc-test-gen analyze \
  --project-path ./my-trpc-project \
  --output analysis.json

# 生成测试用例
./target/release/trpc-test-gen generate \
  --project-path ./my-trpc-project \
  --output-dir ./generated_tests \
  --model gpt-4 \
  --api-key YOUR_API_KEY

# 验证测试覆盖率
./target/release/trpc-test-gen validate \
  --project-path ./my-trpc-project \
  --test-path ./generated_tests
```

详细使用说明请参考: [`crates/trpc-test-generator/README.md`](crates/trpc-test-generator/README.md)

## 🧪 测试

### 编译测试
```bash
cargo check
cargo test
```

### API测试
```bash
# 运行API文档测试
./test_swagger_api_docs.sh

# 运行模型配置测试
./test_model_config_compile.sh
```

## 📝 开发说明

### 添加新API端点
1. 在 `ee/tabby-webserver/src/routes/` 中创建路由文件
2. 添加 `#[utoipa::path]` 注解用于OpenAPI文档
3. 在 `routes/mod.rs` 中注册路由
4. 在 `lib.rs` 的 `EEApiDoc` 中添加路径和schema

### 数据库迁移
```bash
# 创建新迁移
sqlx migrate add migration_name

# 运行迁移
sqlx migrate run
```

## 🤝 贡献

欢迎提交Issue和Pull Request！

## 📄 许可证

本项目采用Apache 2.0许可证。

---

**注意**: 这是一个企业级AI代码助手项目，包含完整的用户模型配置功能和RESTful API接口。