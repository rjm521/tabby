# Tabby 项目开发环境

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
- **获取文档列表**: `GET /v1/index/documents/{corpus}`
- **创建索引**: `POST /v1/index/create`

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

- ✅ 2024-12-19: 解决所有编译问题，`make dev-build` 现在可以正常工作
- ✅ 2024-12-19: 修复 TantivyDocument 类型问题和 Document trait 导入
- ✅ 2024-12-19: 修复 IndexingProgress Clone 问题和 CodeIndexer 方法缺失
- ✅ 2024-12-19: 添加故障排除文档和编译问题解决记录
- ✅ 2024-05-26: 成功安装和配置 Oh My Zsh
- ✅ 2024-05-26: 配置常用别名 (ll, gcmt 等)
- ✅ 2024-05-26: 设置 zsh 为默认 shell

## 许可证

详见 LICENSE 文件。