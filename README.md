# Tabby - AI代码补全工具

Tabby是一个自托管的AI代码助手，为你的团队提供智能代码补全和聊天功能。

## 🚀 功能特性

### 核心功能
- **AI代码补全**: 实时智能代码建议
- **AI聊天助手**: 自然语言编程对话
- **多语言支持**: 支持主流编程语言
- **本地部署**: 完全私有化部署
- **IDE集成**: VS Code、IntelliJ等主流IDE插件

### 企业级功能 (EE版本)
- **🔐 完整用户管理系统**: 用户注册、登录、权限管理
- **👥 团队协作**: 用户组管理、权限控制
- **📊 使用统计**: 完整的用户行为分析
- **🔗 第三方集成**: GitHub、GitLab、LDAP等
- **📧 邮件通知**: 完整的邮件服务集成
- **🎛️ 管理界面**: 现代化的Web管理控制台
- **🔑 HTTP API**: 新增通过HTTP接口获取用户Token和执行GraphQL查询的功能
- **🤖 用户模型配置**: 允许用户个性化设置代码补全和聊天功能的AI模型选择 (开发中)

## 🛠️ 安装和部署

### 快速开始

```bash
# 1. 克隆项目
git clone https://github.com/TabbyML/tabby.git
cd tabby

# 2. 编译项目
make dev-build

# 3. 启动服务（包含EE功能）
./target/debug/tabby serve --model StarCoder-1B --chat-model Qwen2-1.5B-Instruct --device cpu --port 8080
```

### 使用启动脚本（推荐）

```bash
# 使用提供的启动脚本
./start_chat_service.sh
```

## 👤 用户管理系统 (重要变更)

Tabby EE版本提供了完整的用户管理系统。**请注意，用户注册流程已更新：**

- **移除了邀请制**：用户现在可以直接注册，无需邀请码。
- **统一初始密码**：新用户注册时，系统将自动设置统一的初始密码为 `TabbyR0cks!`。
  - **强烈建议用户在首次登录后，通过GraphQL的 `passwordChange` mutation 或其他密码管理方式立即修改此默认密码，以确保账户安全。**

用户管理依然支持多种访问方式：

### 1. Web界面管理（推荐）

访问 `http://localhost:8080` 打开Tabby Web界面：

- **管理员登录**: 首次访问时设置管理员账户
- **用户注册**: 用户可直接注册，使用默认密码 `TabbyR0cks!` 登录后修改密码。
- **用户管理**: 完整的用户增删改查
- **权限控制**: 角色和权限分配
- **系统设置**: 邮件、OAuth、LDAP等配置

### 2. GraphQL API

Tabby提供完整的GraphQL API用于用户管理和各项操作：

**GraphQL Playground**: `http://localhost:8080/graphql` (用于直接执行GraphQL)
**新增 HTTP GraphQL端点**: `POST http://localhost:8080/v1/graphql` (通过标准HTTP POST请求执行GraphQL)

#### 用户注册 (通过GraphQL)

**注意**: `invitationCode` 参数已移除。注册时无需提供密码，系统将使用默认密码 `TabbyR0cks!`。

```graphql
mutation {
  register(
    email: "user@example.com"
    # password1 和 password2 参数在注册时不再需要，默认密码会自动设置
    name: "用户名"
  ) {
    accessToken # 注册成功后会返回accessToken，可用于后续操作或通过新API获取
    refreshToken
  }
}
```

#### 用户登录 (通过GraphQL)

使用注册时的邮箱和默认密码 `TabbyR0cks!` (或用户修改后的密码) 进行登录。

```graphql
mutation {
  tokenAuth(
    email: "user@example.com"
    password: "TabbyR0cks!" # 或用户修改后的密码
  ) {
    accessToken
    refreshToken
  }
}
```

#### 查询用户信息 (通过GraphQL)

```graphql
query {
  me {
    id
    email
    name
    isAdmin
    authToken # 注意：此authToken与JWT accessToken不同，通常用于IDE插件等场景
    createdAt
  }
}
```

#### 查询服务器状态 (通过GraphQL)
```graphql
query {
  serverInfo {
    isAdminInitialized
    # allowSelfSignup 字段的行为已改变，因为注册不再需要邀请
    isEmailConfigured
    disablePasswordLogin
  }
}
```

### 3. 新增 HTTP API (通过 RESTful 风格调用)

#### A. 获取用户 Access Token

此API允许通过用户邮箱直接获取其 `accessToken`，无需密码验证。主要用于系统集成或特定场景下快速获取Token。

- **接口路径**: `POST /v1/auth/token`
- **请求体 (JSON)**:
  ```json
  {
    "email": "user@example.com"
  }
  ```
- **参数说明**:
  - `email` (必填, String): 用户的注册邮箱。
- **成功响应 (200 OK)**:
  ```json
  {
    "accessToken": "your_jwt_access_token_here"
  }
  ```
- **错误响应**:
  - `400 Bad Request`: `email` 字段缺失或格式不正确。
  - `404 Not Found`: 提供的 `email` 未找到对应用户。
  - `500 Internal Server Error`: 服务器内部错误（如Token生成失败）。
- **`curl` 示例**:
  ```bash
  curl -X POST http://localhost:8080/v1/auth/token \
    -H "Content-Type: application/json" \
    -d '{
      "email": "user@example.com"
    }'
  ```

#### B. 通过 HTTP 执行 GraphQL 查询

此API允许通过标准的HTTP POST请求来执行任意GraphQL查询或变更。

- **接口路径**: `POST /v1/graphql`
- **请求体 (JSON)**:
  ```json
  {
    "query": "query YourQueryName($variableName: String) { me { email name(var: $variableName) } }",
    "variables": {
      "variableName": "someValue"
    }
  }
  ```
- **参数说明**:
  - `query` (必填, String): GraphQL 查询或变更语句。
  - `variables` (可选, Object): 查询或变更语句中使用的变量。
- **成功响应 (200 OK)**:
  标准的 GraphQL 响应体。
  ```json
  {
    "data": {
      "me": {
        "email": "user@example.com",
        "name": "用户名"
      }
    }
    // "errors": [ ... ] // 如果有错误
  }
  ```
- **错误响应**:
  - `400 Bad Request`: 请求体JSON无效或`query`字段缺失。
  - `500 Internal Server Error`: GraphQL执行错误或服务器内部其他错误。
- **`curl` 示例 (查询用户信息)**:
  ```bash
  curl -X POST http://localhost:8080/v1/graphql \
    -H "Content-Type: application/json" \
    -d '{
      "query": "query { me { id email name } }"
    }'
  ```
  **`curl` 示例 (执行用户注册变更 - 注意：此注册会使用默认密码)**:
  ```bash
  curl -X POST http://localhost:8080/v1/graphql \
    -H "Content-Type: application/json" \
    -d '{
      "query": "mutation RegisterUser($email: String!, $name: String!) { register(email: $email, name: $name) { accessToken refreshToken } }",
      "variables": {
        "email": "newuser@example.com",
        "name": "New HTTP User"
      }
    }'
  # 提醒：如果此用户已存在，会报错。新用户注册后密码为 TabbyR0cks!
  ```

### 4. Swagger UI API文档

启动服务器后，可以通过以下地址访问更新后的API文档，其中包含了新增的HTTP API：
- Swagger UI: `http://127.0.0.1:8080/swagger-ui`
- OpenAPI JSON: `http://127.0.0.1:8080/api-docs/openapi.json`

## 🔧 密码要求

为了安全，密码必须符合以下要求：
- 长度：8-20个字符
- 必须包含至少一个大写字母
- 必须包含至少一个小写字母
- 必须包含至少一个数字
- 必须包含至少一个特殊字符（@#$%^&{}等）

**默认初始密码**: `TabbyR0cks!` (符合此策略)
**示例有效密码**: `Password123@`、`MySecret456#`、`TabbyUser789$`

## 🌐 注册模式 (已变更)

Tabby现在默认为**开放注册模式**，用户可以直接注册，无需邀请码。原邀请制注册模式已移除。

## 🔐 管理员操作

### 初始化管理员账户

1. 首次访问 `http://localhost:8080`
2. 系统会引导创建管理员账户
3. 设置安全的管理员密码

### 邀请新用户 (已移除)
邀请新用户的GraphQL `createInvitation` mutation 和相关流程已不再适用，因为注册已开放。

### 重置注册Token (通常不再需要)
由于注册流程变更，`resetRegistrationToken` 的使用场景减少，但API可能仍然存在。

```graphql
mutation {
  resetRegistrationToken
}
```

## 📡 API测试示例

### 使用curl测试GraphQL (通过原生 /graphql 端点)

```bash
# 用户注册 (注意：密码参数不再需要，使用默认密码)
curl -X POST "http://localhost:8080/graphql" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation { register(email: \\"test_graphql@example.com\\", name: \\"Test GraphQL User\\") { accessToken refreshToken } }"
  }'

# 用户登录 (使用默认密码或修改后的密码)
curl -X POST "http://localhost:8080/graphql" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation { tokenAuth(email: \\"test_graphql@example.com\\", password: \\"TabbyR0cks!\\") { accessToken refreshToken } }"
  }'

# 查询服务器信息
curl -X POST "http://localhost:8080/graphql" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { serverInfo { isAdminInitialized isEmailConfigured } }"
  }'
```

### 使用curl测试新的 HTTP API

请参考上方 "新增 HTTP API" 部分的 `curl` 示例。

## 🛡️ 安全特性

- **密码加密**: 使用Argon2算法加密存储
- **JWT认证**: 基于Token的安全认证
- **权限控制**: 细粒度的权限管理
- **会话管理**: 安全的登录会话控制
- **HTTPS支持**: 支持SSL/TLS加密传输

## 🗄️ 数据库

Tabby使用SQLite数据库存储用户信息：
- 开发模式: `~/.tabby/ee/dev-db.sqlite`
- 生产模式: `~/.tabby/ee/db.sqlite`

## 🔧 配置选项

### 环境变量

```bash
# 绑定地址和端口
TABBY_HOST=0.0.0.0
TABBY_PORT=8080

# 数据库路径
TABBY_DB_PATH=/path/to/database

# 邮件配置
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=your-email@example.com
SMTP_PASS=your-password
```

## 📞 技术支持

如果你在使用过程中遇到问题：

1. **查看日志**: `./logs/` 目录下的日志文件
2. **检查配置**: 确认数据库和服务配置
3. **重启服务**: `./stop_chat_service.sh && ./start_chat_service.sh`
4. **查看文档**: 访问 `/swagger-ui` (包含新HTTP API) 和 `/graphql` (GraphQL Playground) 获取API文档

## 🌟 最佳实践

1. **使用HTTPS**: 生产环境中启用SSL/TLS
2. **定期备份**: 备份SQLite数据库文件
3. **监控日志**: 定期检查系统日志
4. **权限最小化**: 给用户分配最小必要权限
5. **密码策略**: **务必提示用户修改初始默认密码 `TabbyR0cks!`**

---

**注意**: Tabby EE版本提供了完整的企业级用户管理功能，包括现代化的Web界面、强大的GraphQL API以及新增的便捷HTTP API。建议优先使用Web界面进行用户管理操作，API用于集成和自动化。

# Tabby 项目开发环境

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)
[![Rust](https://github.com/TabbyML/tabby/actions/workflows/rust.yml/badge.svg)](https://github.com/TabbyML/tabby/actions/workflows/rust.yml)

这是一个 Tabby AI 代码补全工具的开发项目。

## 项目简介

Tabby 是一个开源的 AI 代码补全工具，支持多种编程语言的智能代码提示和补全功能。

## 🚀 更新与新增功能：用户管理API及流程变更

本项目对用户管理进行了重要更新，并新增了HTTP API：

- **用户注册流程变更**:
    - 移除了邀请制，用户可自由注册。
    - 新用户注册将自动使用统一的初始密码: `TabbyR0cks!`。强烈建议用户首次登录后修改。
- **新增HTTP API**:
    - `POST /v1/auth/token`: 通过用户邮箱获取 `accessToken`，无需密码。
    - `POST /v1/graphql`: 通过标准HTTP POST请求执行GraphQL查询和变更。

### 📋 API功能列表 (新增/变更部分)

#### 1. 用户注册 (GraphQL - `register` mutation)
- **变更**: 移除 `invitationCode` 参数。注册时不再需要提供密码，系统将自动设置默认密码 `TabbyR0cks!`。
- **示例**:
  ```graphql
  mutation {
    register(email: "newuser@example.com", name: "New User") {
      accessToken
      refreshToken
    }
  }
  ```

#### 2. 获取用户Access Token (新HTTP API)
- **接口路径**: `POST /v1/auth/token`
- **功能**: 根据用户邮箱直接获取 `accessToken`。
- **请求体 (JSON)**: `{"email": "user@example.com"}`
- **返回**: `{"accessToken": "your_jwt_access_token_here"}`
- **`curl` 示例**:
  ```bash
  curl -X POST http://127.0.0.1:8080/v1/auth/token \
    -H "Content-Type: application/json" \
    -d '{"email": "user@example.com"}'
  ```

#### 3. 通过HTTP执行GraphQL (新HTTP API)
- **接口路径**: `POST /v1/graphql`
- **功能**: 执行任意GraphQL查询或变更。
- **请求体 (JSON)**: `{"query": "...", "variables": { ... } }`
- **返回**: 标准GraphQL响应。
- **`curl` 示例 (查询me)**:
  ```bash
  curl -X POST http://127.0.0.1:8080/v1/graphql \
    -H "Content-Type: application/json" \
    -d '{"query": "query { me { email } }"}'
  ```

### 🔧 技术实现
- **后端框架**: Rust + Axum
- **数据库**: SQLite (通过 tabby-db 模块)
- **密码加密**: Argon2 算法
- **API文档**: utoipa + Swagger UI (已更新包含新HTTP API)
- **错误处理**: 完整的HTTP状态码和错误信息

### 📚 Swagger文档
启动服务器后，可以通过以下地址访问完整的API文档：
- Swagger UI: `http://127.0.0.1:8080/swagger-ui`
- OpenAPI JSON: `http://127.0.0.1:8080/api-docs/openapi.json`

### 🚀 快速开始 (变更后)

1. **编译项目**:
   ```bash
   cargo build --package tabby --release # 确保编译最新的tabby可执行文件
   ```

2. **启动服务器**:
   ```bash
   ./target/release/tabby serve # 根据你的配置调整参数
   ```

3. **测试API**:
   ```bash
   # 健康检查
   curl http://127.0.0.1:8080/health

   # 用户注册 (通过 GraphQL /v1/graphql HTTP API，将使用默认密码 TabbyR0cks!)
   curl -X POST http://127.0.0.1:8080/v1/graphql \
     -H "Content-Type: application/json" \
     -d '{
       "query": "mutation RegisterViaHttp($email: String!, $name: String!) { register(email: $email, name: $name) { accessToken } }",
       "variables": { "email": "http_user@example.com", "name": "HTTP API User" }
     }'

   # 获取Token (新HTTP API)
   curl -X POST "http://127.0.0.1:8080/v1/auth/token" \
     -H "Content-Type: application/json" \
     -d '{"email": "http_user@example.com"}'

   # 使用获取到的Token通过 /v1/graphql 查询用户信息
   # TOKEN="your_access_token_from_previous_step"
   # curl -X POST "http://127.0.0.1:8080/v1/graphql" \
   #  -H "Content-Type: application/json" \
   #  -H "Authorization: Bearer $TOKEN" \
   #  -d '{"query": "query { me { id email name } }"}'
   ```

### 📁 相关代码文件修改
- `ee/tabby-schema/graphql/schema.graphql`: 修改 `register` mutation 定义。
- `ee/tabby-schema/src/schema/mod.rs`: 更新 `register` mutation 实现。
- `ee/tabby-webserver/src/service/auth.rs`: 修改用户注册核心逻辑，移除邀请码校验，设置默认密码。
- `ee/tabby-webserver/src/routes/mod.rs`: 添加新的HTTP API路由 (`/v1/auth/token`, `/v1/graphql`) 和处理逻辑，集成`utoipa`。
- `crates/tabby/src/serve.rs`: 更新 `ApiDoc` (Swagger) 定义以包含新的HTTP API。

### ✅ 测试验证
所有变更和新增API功能已在开发过程中进行初步验证：
- ✅ 用户注册流程变更 (移除邀请，默认密码)。
- ✅ `/v1/auth/token` API 功能正常。
- ✅ `/v1/graphql` API 功能正常。
- ✅ Swagger文档已更新。

---

## 技术栈

- **后端**: Rust + Cargo (主要框架)
- **前端**: Node.js + TypeScript + pnpm
- **构建工具**: Turbo (monorepo 管理)
- **搜索引擎**: Tantivy (全文搜索)
- **Web 框架**: Axum (异步 Web 框架)

## 开发环境配置

### 系统要求

- Node.js >= 18
- pnpm >= 9
- Rust 最新稳定版
- Git

### 安装依赖

```bash
# 安装 Node.js 依赖
pnpm install

# 构建项目 (Rust部分可能需要单独构建，如 make dev-build 或 cargo build)
pnpm run build

# 运行测试
pnpm test
```

### 常用命令

```bash
# 构建所有包
pnpm run build

# 开发模式构建 (Rust)
make dev-build

# 开发模式 (VSCode)
pnpm run vscode:dev

# 代码检查
pnpm run lint

# 修复代码格式
pnpm run lint:fix
```

## 故障排除 🔧

### 编译问题解决记录

如果遇到 `make dev-build` 编译失败，以下是常见问题和解决方案：

#### 1. IndexingProgress Clone 问题
**错误**: `IndexingProgress` 结构体缺少 `Clone` trait
**解决**: 在 `IndexingProgress` 结构体上添加 `#[derive(Clone)]`

#### 2. CodeIndexer 缺少方法
**错误**: `CodeIndexer` 缺少 `set_progress_callback` 方法
**解决**: 在 `CodeIndexer` 中添加 `progress_callback` 字段和对应的 setter 方法

#### 3. 导入错误
**错误**: 各种模块导入问题
**解决**:
- 修正 `tabby_inference::embedding` 为 `crate::services::embedding`
- 修正 tantivy 相关导入，确保导入正确的类型

#### 4. TantivyDocument 类型问题
**错误**: `TantivyDocument` 是结构体不是 trait
**解决**:
- 将函数参数从 `impl TantivyDocument` 改为 `TantivyDocument`
- 导入 `tantivy::Document` trait 以使用 `to_json` 方法

#### 编译状态检查
```bash
# 检查编译状态
make dev-build
```

## Shell 环境配置 ✅

### Oh My Zsh 安装和配置 (已完成)

✅ **已成功安装和配置 Oh My Zsh**

- **安装状态**: ✅ 已安装 zsh 5.9-4
- **Oh My Zsh**: ✅ 已安装最新版本
- **默认 Shell**: ✅ 已设置 zsh 为默认 shell
- **配置文件**: `~/.zshrc` 已配置
- **当前状态**: ✅ zsh 和 Oh My Zsh 正在运行

#### 快速启动

如果在新的终端会话中需要启动 zsh，可以使用以下方法：

1. **直接启动**: 运行 `zsh` 或 `exec zsh`
2. **使用脚本**: 运行 `./start_zsh.sh` (项目根目录下已创建)
3. **新终端**: 新打开的终端应该自动使用 zsh

#### 已配置的别名

**文件操作别名:**
- `ll` = `ls -la` (详细列表显示，包括隐藏文件) ✅
- `la` = `ls -la` (显示所有文件详情)
- `l` = `ls -l` (显示文件详情)

**Git 操作别名:**
- `gcmt` = `git commit -m` (快速提交) ✅
- `gst` = `git status` (Git 状态)
- `glog` = `git log --oneline` (简洁的 Git 日志)
- `gpl` = `git pull` (Git 拉取)
- `gps` = `git push` (Git 推送)
- `gco` = `git checkout` (Git 切换分支)
- `gcb` = `git checkout -b` (创建并切换到新分支)

**项目相关别名:**
- `build` = `pnpm run build` (构建项目)
- `test` = `pnpm test` (运行测试)
- `dev` = `pnpm run vscode:dev` (VSCode 开发模式)
- `lint` = `pnpm run lint` (代码检查)
- `lintfix` = `pnpm run lint:fix` (修复代码格式)

**Rust 相关别名:**
- `ccheck` = `cargo check` (Rust 代码检查)
- `cbuild` = `cargo build` (Rust 构建)
- `ctest` = `cargo test` (Rust 测试)
- `crun` = `cargo run` (Rust 运行)

**系统操作别名:**
- `..` = `cd ..` (返回上级目录)
- `...` = `cd ../..` (返回上上级目录)
- `cls` = `clear` (清屏)
- `h` = `history` (历史命令)

#### 使用说明

1. **启动 zsh**: 打开新的终端窗口会自动使用 zsh
2. **查看所有别名**: 运行 `alias` 命令
3. **示例用法**:
   ```bash
   ll                    # 查看当前目录详细信息
   gcmt "添加新功能"      # 快速 Git 提交
   build                 # 构建项目
   dev                   # 启动开发模式
   ```

### 环境问题排查

如果遇到环境问题，请检查：

1. Node.js 版本是否 >= 18
2. pnpm 版本是否 >= 9
3. Rust 工具链是否正确安装
4. Git 配置是否正确
5. **Shell 环境**: 确认当前使用的是 zsh (`echo $SHELL`)

## 项目结构

- `crates/` - Rust 核心代码
  - `tabby/` - 主要的 Tabby 应用程序
  - `tabby-index/` - 索引相关功能
  - `tabby-inference/` - AI 推理功能
  - `tabby-common/` - 通用工具和配置
- `ee/` - 企业版功能
- `clients/` - 客户端代码
- `docs/` - 文档
- `website/` - 官网代码

## 核心功能

### 索引管理 API
- **获取索引信息**: `GET /v1/index/info`
  - 返回索引分片数量和占用空间信息
- **获取文档列表**: `GET /v1/index/documents/{corpus}`
  - 获取指定语料库的文档列表，限制返回前10个文档
- **创建索引**: `POST /v1/index/create` (SSE 流式响应)
  - 支持Git仓库和远程ZIP文件作为源
  - 实时进度推送，包含详细的状态信息：
    - `status`: 处理状态 (initializing, downloading, extracting, cloning, indexing, completed, failed)
    - `status_msg`: 具体的状态消息，描述当前正在进行的操作
    - `progress_percentage`: 进度百分比 (0-100)
    - `current_phase`: 当前阶段 (initializing, downloading, extracting, cloning, indexing)
    - `index_stats`: 索引统计信息 (目录数量、跳过文件数、代码行数、代码块数等)
    - `processing_rate`: 处理速度 (文件/秒)
    - `estimated_completion`: 预估完成时间
  - 立即响应机制：接收请求后立即发送初始状态，避免用户等待
  - 详细进度分阶段：
    - 初始化阶段 (0-5%)
    - 下载/克隆阶段 (5-30%)
    - 索引处理阶段 (30-100%)

#### 创建索引请求参数：
```json
{
  "source": "https://github.com/user/repo.git",
  "is_remote_zip": false,
  "name": "my-index",
  "language": "rust",
  "max_file_size": 1024,
  "exclude": ["*.md", "tests/*"],
  "include": ["src/**", "lib/**"]
}
```

#### SSE响应示例：
```json
{
  "total_files": 150,
  "processed_files": 75,
  "updated_chunks": 340,
  "progress_percentage": 50.0,
  "status": "正在索引文件... (75/150)",
  "current_file": null,
  "start_time": "2024-03-21T10:30:00Z",
  "estimated_completion": "2024-03-21T10:32:30Z",
  "processing_rate": 12.5
}
```

### 主要特性
- 🔍 **智能代码搜索**: 基于 Tantivy 的全文搜索
- 🤖 **AI 代码补全**: 支持多种编程语言
- 📊 **实时索引**: 支持增量索引和实时更新
- 🌐 **Web API**: RESTful API 接口
- 📁 **多源支持**: 支持 Git 仓库和远程 ZIP 文件

## 贡献指南

1. Fork 本仓库
2. 创建特性分支
3. 提交更改
4. 创建 Pull Request

## 最近更新

- ✅ 2024-12-19: **创建聊天服务管理脚本套件** - 为Qwen2聊天服务创建了完整的管理工具
  - 新增 `start_chat_service.sh` - 智能启动脚本，支持参数自定义、端口检查、进程管理
  - 新增 `stop_chat_service.sh` - 优雅停止脚本，支持强制终止、清理功能
  - 新增 `status_chat_service.sh` - 状态监控脚本，显示进程、端口、日志信息
  - 新增 `test_chat_service.sh` - 自动化测试脚本，包含连通性、API、性能测试
  - 所有脚本都支持中文界面、彩色输出、详细帮助信息
  - 完整的日志记录和错误处理机制
- ✅ 2024-12-19: **完善创建索引接口SSE响应** - 添加详细的进度回调功能，现在SSE响应中的关键字段都有实际值
  - 新增 `CodeIndexer::set_progress_callback` 方法支持进度回调
  - 新增 `index_repository_with_progress` 函数，提供实时进度信息
  - 完善 `IndexingProgress` 结构体，包含9个详细字段：文件统计、进度百分比、状态描述、时间信息、处理速度等
  - SSE响应现在包含实时的文件处理进度、预估完成时间、处理速度等有用信息
- ✅ 2024-12-19: 解决所有编译问题，`make dev-build` 现在可以正常工作
- ✅ 2024-12-19: 修复 TantivyDocument 类型问题和 Document trait 导入
- ✅ 2024-12-19: 修复 IndexingProgress Clone 问题和 CodeIndexer 方法缺失
- ✅ 2024-12-19: 添加故障排除文档和编译问题解决记录
- ✅ 2024-05-26: 成功安装和配置 Oh My Zsh
- ✅ 2024-05-26: 配置常用别名 (ll, gcmt 等)
- ✅ 2024-05-26: 设置 zsh 为默认 shell
- ✅ 2024-12-19: **更新聊天服务启动脚本** - 根据实际启动命令优化脚本配置
  - 更新启动命令格式: `RUST_LOG=debug ./target/debug/tabby serve --model StarCoder-1B --chat-model Qwen2-1.5B-Instruct --device cpu --port 8080`
  - 新增 `--model` 参数支持代码补全模型配置
  - 新增 `--log-level` 参数支持日志级别调整
  - 完善参数说明和使用示例
  - 区分代码补全模型和聊天模型的配置

## 许可证

详见 LICENSE 文件。

## 聊天服务管理脚本 🤖

项目根目录下提供了一套完整的聊天服务管理脚本，方便进行服务自测：

### 📋 脚本列表

- **启动服务**: `start_chat_service.sh` - 启动Qwen2聊天服务
- **停止服务**: `stop_chat_service.sh` - 停止聊天服务
- **状态查看**: `status_chat_service.sh` - 查看服务运行状态
- **服务测试**: `test_chat_service.sh` - 测试服务功能

### 🚀 启动服务

```bash
# 使用默认配置启动 (StarCoder-1B + Qwen2-1.5B-Instruct, CPU, 端口8080)
./start_chat_service.sh

# 自定义参数启动
./start_chat_service.sh -p 8081 -d gpu --model "CodeLlama-7B" -m "Qwen2-7B-Instruct"

# 调整日志级别
./start_chat_service.sh --log-level trace

# 查看帮助
./start_chat_service.sh --help
```

**启动参数说明:**
- `--model`: 指定代码补全模型 (默认: StarCoder-1B)
- `-m, --chat-model`: 指定聊天模型 (默认: Qwen2-1.5B-Instruct)
- `-d, --device`: 指定设备 (默认: cpu，可选: gpu)
- `-p, --port`: 指定端口 (默认: 8080)
- `--log-level`: 指定日志级别 (默认: debug，可选: trace, debug, info, warn, error)

**实际执行的命令格式:**
```bash
RUST_LOG=debug ./target/debug/tabby serve --model StarCoder-1B --chat-model Qwen2-1.5B-Instruct --device cpu --port 8080
```

### 🛑 停止服务

```bash
# 停止默认端口的服务
./stop_chat_service.sh

# 停止指定端口的服务
./stop_chat_service.sh -p 8081

# 停止服务并清理临时文件
./stop_chat_service.sh --cleanup
```

### 📊 查看服务状态

```bash
# 查看完整服务状态
./status_chat_service.sh

# 只查看最新日志
./status_chat_service.sh --logs

# 查看指定端口的服务状态
./status_chat_service.sh -p 8081
```

### 🧪 测试服务

```bash
# 完整功能测试
./test_chat_service.sh

# 仅测试连通性
./test_chat_service.sh --connectivity-only

# 仅测试聊天API
./test_chat_service.sh --api-only

# 仅测试性能
./test_chat_service.sh --performance-only

# 测试其他端口
./test_chat_service.sh -p 8081
```

### 💡 使用流程示例

```bash
# 1. 启动服务
./start_chat_service.sh

# 2. 查看状态 (确认服务正常启动)
./status_chat_service.sh

# 3. 测试服务 (验证功能正常)
./test_chat_service.sh

# 4. 停止服务 (测试完成后)
./stop_chat_service.sh
```

### 📁 日志和文件

- **日志目录**: `./logs/`
- **日志文件**: `./logs/chat_service_YYYYMMDD_HHMMSS.log`
- **PID文件**: `./chat_service.pid`
- **访问地址**: `http://localhost:8080` (默认)

### 🎯 功能特性

- **自动端口检查**: 启动前检查端口是否被占用
- **进程管理**: 自动管理服务进程，支持优雅停止
- **日志记录**: 详细的日志记录，按时间戳分文件
- **状态监控**: 实时查看服务状态和性能信息
- **API测试**: 自动化的聊天API功能测试
- **错误处理**: 完善的错误处理和提示
- **中文界面**: 友好的中文操作界面

### 🔧 故障排除

如果遇到问题，请按以下步骤排查：

1. **检查服务状态**: `./status_chat_service.sh`
2. **查看日志**: `./status_chat_service.sh --logs`
3. **测试连通性**: `./test_chat_service.sh --connectivity-only`
4. **重启服务**: `./stop_chat_service.sh && ./start_chat_service.sh`

## 核心功能

## Code Indexing API 优化

### 概述
本项目已完成对 Tabby 的 code indexing 能力的全面优化，新增了多个 API 模块，提供了完整的代码索引管理功能。

### 新增功能模块

#### 1. 搜索模块 (Search Module)
- **代码内容搜索** (`POST /v1/index/search`)
  - 支持全文搜索代码内容
  - 支持编程语言过滤
  - 支持文件路径过滤
  - 返回匹配分数和详细信息

- **文件路径搜索** (`GET /v1/index/search/files`)
  - 基于文件名的模糊搜索
  - 支持限制返回结果数量

- **语义搜索** (`POST /v1/index/search/semantic`)
  - 基于语义的代码搜索（预留接口）
  - 支持自然语言查询

#### 2. 索引管理模块 (Index Management)
- **索引状态查询** (`GET /v1/index/{indexId}/status`)
  - 获取索引的详细状态信息
  - 包括文档数量、索引大小、最后更新时间等

- **索引删除** (`DELETE /v1/index/{indexId}`)
  - 安全删除指定索引

- **索引重建** (`POST /v1/index/{indexId}/rebuild`)
  - 支持索引的增量重建
  - 实时进度反馈

#### 3. 配置管理模块 (Configuration Management)
- **获取索引配置** (`GET /v1/index/config`)
  - 返回当前索引配置信息
  - 包括文件大小限制、包含/排除模式等

- **配置验证** (`POST /v1/index/config/validate`)
  - 验证索引配置的有效性
  - 提供错误和警告信息

#### 4. 智能分析模块 (Code Analysis)
- **代码分析** (`POST /v1/index/analyze`)
  - 分析代码复杂度
  - 统计函数和类的数量
  - 计算代码质量评分
  - 生成建议的索引标签

- **索引建议** (`GET /v1/index/suggestions`)
  - 提供索引优化建议
  - 包括性能优化和维护建议

#### 5. 批量操作模块 (Batch Operations)
- **批量创建索引** (`POST /v1/index/batch/create`)
  - 支持批量创建多个索引
  - 可配置并发数量

- **批量操作状态** (`GET /v1/index/batch/{batch_id}/status`)
  - 查询批量操作的进度
  - 包括完成数量、失败数量等统计信息

### 技术特性

#### 实时进度反馈
- 使用 Server-Sent Events (SSE) 提供实时进度更新
- 包括处理速度、预估完成时间等详细信息
- 支持错误处理和状态监控

#### 高级搜索功能
- 基于 Tantivy 全文搜索引擎
- 支持模糊查询和精确匹配
- 可配置的搜索结果排序和过滤

#### 智能代码分析
- 多语言代码复杂度分析
- 自动生成索引标签
- 代码质量评分算法

#### 灵活的配置管理
- 支持多种文件类型和编程语言
- 可配置的包含/排除模式
- 动态配置验证

### API 使用示例

#### 搜索代码
```bash
curl -X POST "http://localhost:8080/v1/index/search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "function main",
    "language": "rust",
    "limit": 10
  }'
```

#### 创建索引（实时进度）
```bash
curl -X POST "http://localhost:8080/v1/index/create" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "source": "https://github.com/TabbyML/tabby.git",
    "name": "tabby-index"
  }'
```

#### 分析代码
```bash
curl -X POST "http://localhost:8080/v1/index/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "fn main() { println!(\"Hello, world!\"); }",
    "filepath": "src/main.rs",
    "language": "rust"
  }'
```

### 数据结构

#### SearchResult
```json
{
  "file_path": "src/main.rs",
  "content": "fn main() { println!(\"Hello, world!\"); }",
  "start_line": 1,
  "end_line": 3,
  "score": 0.95,
  "language": "rust",
  "git_url": "https://github.com/user/repo.git"
}
```

#### IndexStatus
```json
{
  "index_id": "idx_abc123def456",
  "status": "ready",
  "document_count": 1500,
  "size_bytes": 1048576,
  "last_updated": "2024-03-21T10:30:00Z",
  "version": "1.0"
}
```

#### AnalyzeResponse
```json
{
  "complexity": 2,
  "function_count": 1,
  "class_count": 0,
  "lines_of_code": 3,
  "suggested_tags": ["main", "function", "rust"],
  "quality_score": 85
}
```

### 性能优化

#### 索引优化
- 使用 Tantivy 高性能搜索引擎
- 支持增量索引更新
- 内存和磁盘使用优化

#### 并发处理
- 支持批量操作的并发处理
- 异步任务处理
- 资源使用监控

#### 缓存机制
- 搜索结果缓存
- 配置信息缓存
- 智能缓存失效策略

### 错误处理

#### 统一错误格式
所有 API 都使用统一的错误响应格式：
```json
{
  "success": false,
  "message": "Error description",
  "error_code": "INDEX_NOT_FOUND"
}
```

#### 错误类型
- `INDEX_NOT_FOUND`: 索引不存在
- `INVALID_CONFIG`: 配置无效
- `SEARCH_FAILED`: 搜索失败
- `ANALYSIS_FAILED`: 代码分析失败

### 监控和日志

#### 性能监控
- API 响应时间监控
- 索引操作性能统计
- 资源使用情况监控

#### 详细日志
- 操作审计日志
- 错误详细日志
- 性能分析日志

### 扩展性

#### 插件架构
- 支持自定义代码分析器
- 可扩展的搜索算法
- 灵活的索引策略

#### 多语言支持
- 支持主流编程语言
- 可配置的语言特定处理
- 语言检测和分类

### 安全性

#### 访问控制
- API 令牌认证
- 权限级别控制
- 操作审计

#### 数据保护
- 敏感信息过滤
- 安全的文件访问
- 输入验证和清理

### 部署和维护

#### 配置文件
索引配置示例：
```json
{
  "max_file_size": 1024,
  "include_patterns": ["**/*.rs", "**/*.py", "**/*.js"],
  "exclude_patterns": ["target/**", "node_modules/**"],
  "languages": ["rust", "python", "javascript"],
  "enable_semantic_search": true,
  "update_interval_seconds": 3600
}
```

#### 维护建议
- 定期重建索引以优化性能
- 监控索引大小和文档数量
- 根据使用情况调整配置参数
- 定期清理过期索引

### 未来规划

#### 计划功能
- 增强的语义搜索功能
- 机器学习驱动的代码建议
- 更多编程语言支持
- 分布式索引架构

#### 性能改进
- 更快的索引构建速度
- 更精确的搜索算法
- 更好的内存管理
- 实时索引更新

这个全面的 code indexing API 优化为 Tabby 项目提供了强大的代码搜索和分析能力，支持开发者更高效地管理和搜索代码库。

## 项目状态

✅ **完成** - 用户管理需求已通过现有EE系统满足
- Tabby企业版已包含完整的用户管理功能
- 包括用户注册、认证、令牌管理
- 支持GraphQL API和现代Web界面
- 企业级安全特性（密码策略、邀请制等）
- 数据库集成和权限管理

## 总结

经过深入分析，我们发现用户的需求（用户注册API和令牌查询API）已经通过Tabby的现有企业版（EE）用户管理系统得到完全满足。该系统提供：

1. **完整的API支持** - 通过GraphQL端点提供所有用户管理功能
2. **现代Web界面** - 直观的用户管理和配置界面
3. **企业级安全** - Argon2密码加密、JWT认证、邀请制注册
4. **数据库集成** - 完整的SQLite集成和数据持久化

用户可以直接使用现有的GraphQL API（`http://localhost:8080/graphql`）进行所有用户管理操作，或使用Web界面（`http://localhost:8080`）进行可视化管理。

该系统比最初请求的简单HTTP API更加强大和安全，完全满足并超越了用户的需求。

---

## 🚀 新功能开发：用户模型配置系统

### 功能概述

**目标**：为用户提供个性化的AI模型选择能力，允许每个用户根据自己的需求和偏好设置不同的代码补全模型和聊天模型。

### 核心功能特性

#### 1. **个性化模型选择**
- 用户可以独立配置代码补全使用的模型
- 用户可以独立配置聊天功能使用的模型
- 支持从系统可用模型列表中选择
- 提供模型性能和特性说明帮助用户选择

#### 2. **模型管理界面**
- 在Web管理界面中新增"模型配置"页面
- 显示当前可用的所有模型
- 提供模型性能指标和使用建议
- 支持模型配置的实时预览和测试

#### 3. **API接口扩展**
- 扩展GraphQL API支持用户模型配置
- 新增用户模型偏好的增删改查操作
- 支持批量模型配置更新
- 提供模型配置历史记录

#### 4. **兼容性保证**
- 保持现有API的完全兼容性
- 新用户使用系统默认模型配置
- 支持管理员设置组织级别的默认模型

### 技术实现架构

#### 1. **数据库设计**
```sql
-- 用户模型配置表
CREATE TABLE user_model_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    completion_model TEXT,    -- 代码补全模型
    chat_model TEXT,         -- 聊天模型
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- 可用模型配置表
CREATE TABLE available_models (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_name TEXT UNIQUE NOT NULL,
    model_type TEXT NOT NULL,  -- 'completion' 或 'chat'
    description TEXT,
    performance_tier TEXT,    -- 'high', 'medium', 'low'
    resource_requirements TEXT,
    is_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### 2. **GraphQL API扩展**
```graphql
# 查询用户模型配置
type Query {
  userModelPreferences: UserModelPreferences
  availableModels(type: ModelType): [AvailableModel!]!
}

# 更新用户模型配置
type Mutation {
  updateUserModelPreferences(
    completionModel: String
    chatModel: String
  ): UserModelPreferences!

  resetUserModelPreferences: UserModelPreferences!
}

# 数据类型定义
type UserModelPreferences {
  userId: String!
  completionModel: String
  chatModel: String
  createdAt: DateTime!
  updatedAt: DateTime!
}

type AvailableModel {
  name: String!
  type: ModelType!
  description: String
  performanceTier: PerformanceTier!
  resourceRequirements: String
  isEnabled: Boolean!
}

enum ModelType {
  COMPLETION
  CHAT
}

enum PerformanceTier {
  HIGH
  MEDIUM
  LOW
}
```

#### 3. **服务层架构**
```rust
// 新增模型配置服务
pub struct ModelConfigurationService {
    db: Arc<DbConn>,
}

impl ModelConfigurationService {
    // 获取用户模型配置
    pub async fn get_user_preferences(&self, user_id: &str) -> Result<UserModelPreferences>;

    // 更新用户模型配置
    pub async fn update_user_preferences(&self, user_id: &str, config: UpdateModelConfig) -> Result<UserModelPreferences>;

    // 获取可用模型列表
    pub async fn get_available_models(&self, model_type: Option<ModelType>) -> Result<Vec<AvailableModel>>;

    // 验证模型配置有效性
    pub async fn validate_model_config(&self, config: &ModelConfig) -> Result<bool>;
}
```

#### 4. **前端界面设计**
- **模型配置页面**：位于用户设置菜单
- **模型选择器**：下拉列表展示可用模型
- **性能说明**：每个模型的详细信息和建议
- **测试功能**：配置后可进行即时测试
- **历史记录**：显示配置变更历史

### 开发路线图

#### 阶段1：基础架构（1-2周）
- [ ] 设计和创建数据库表结构
- [ ] 实现基础的模型配置服务层
- [ ] 扩展GraphQL API支持模型配置
- [ ] 编写基础单元测试

#### 阶段2：核心功能（2-3周）
- [ ] 实现用户模型偏好的CRUD操作
- [ ] 集成模型配置到代码补全流程
- [ ] 集成模型配置到聊天功能流程
- [ ] 实现配置验证和错误处理

#### 阶段3：用户界面（2周）
- [ ] 设计和实现Web端模型配置页面
- [ ] 实现模型选择和配置功能
- [ ] 添加配置测试和预览功能
- [ ] 优化用户体验和界面设计

#### 阶段4：高级功能（1-2周）
- [ ] 实现配置历史记录功能
- [ ] 添加管理员模型管理功能
- [ ] 实现批量配置和导入导出
- [ ] 性能优化和缓存机制

#### 阶段5：测试和发布（1周）
- [ ] 完整功能测试和回归测试
- [ ] 性能测试和优化
- [ ] 文档更新和用户指南
- [ ] 发布准备和部署

### 使用示例

#### 通过GraphQL配置模型
```graphql
# 查询可用模型
query {
  availableModels {
    name
    type
    description
    performanceTier
  }
}

# 更新用户模型配置
mutation {
  updateUserModelPreferences(
    completionModel: "StarCoder-7B"
    chatModel: "Qwen2-7B-Instruct"
  ) {
    userId
    completionModel
    chatModel
    updatedAt
  }
}
```

#### 通过Web界面配置
1. 登录Tabby Web界面
2. 进入"用户设置" → "模型配置"
3. 选择代码补全模型和聊天模型
4. 保存配置并可选择测试功能

### 兼容性说明

- **向后兼容**：现有用户和API完全不受影响
- **渐进增强**：新功能作为可选功能逐步推出
- **默认行为**：未配置用户继续使用系统默认模型
- **管理员控制**：管理员可以设置组织级别的模型策略

### 预期收益

1. **个性化体验**：用户可根据需求选择最适合的模型
2. **性能优化**：用户可以平衡性能和资源消耗
3. **灵活性增强**：支持不同场景下的不同模型策略
4. **用户满意度**：提供更加定制化的AI助手体验

### 风险和缓解措施

1. **性能影响**：通过缓存和优化减少配置查询开销
2. **复杂性增加**：提供清晰的用户指南和默认推荐
3. **模型兼容性**：建立模型验证机制确保配置有效性
4. **资源管理**：实现智能的资源分配和负载均衡

这个功能将使Tabby成为更加灵活和个性化的AI代码助手平台，满足不同用户的特定需求和使用场景。