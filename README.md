# Tabby 项目开发环境

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)
[![Rust](https://github.com/TabbyML/tabby/actions/workflows/rust.yml/badge.svg)](https://github.com/TabbyML/tabby/actions/workflows/rust.yml)

这是一个 Tabby AI 代码补全工具的开发项目。

## 项目简介

Tabby 是一个开源的 AI 代码补全工具，支持多种编程语言的智能代码提示和补全功能。

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

# 构建项目
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

# 或者直接使用 cargo
cargo build --bin tabby
```

### 当前编译状态 ✅
- **状态**: ✅ 编译成功
- **警告**: 存在一些未使用的字段和变量警告（不影响功能）
- **最后更新**: 2024-12-19

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