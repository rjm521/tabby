# tRPC 测试生成工具

基于AI驱动的tRPC-Go项目测试用例自动生成工具，类似于qodo-ai/qodo-cover，专门为tRPC-Go项目设计。

## 🎯 功能特性

### 核心功能
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

### 测试生成策略
- 🎯 为每个tRPC路由生成专门的测试用例
- 🛡️ 输入验证和边界条件测试
- ⚠️ 全面的错误处理场景覆盖
- 📋 表驱动测试模式（table-driven tests）
- 🔄 Mock和依赖注入支持

## 🏗️ 项目架构

```
src/
├── main.rs              # 命令行工具入口
├── agent/               # AI代理模块
│   ├── mod.rs           # Agent接口定义
│   ├── completion.rs    # LLM交互（rig框架集成）
│   └── trpc_analyzer.rs # tRPC代码分析器
├── generator/           # 测试代码生成器
│   ├── mod.rs           # 生成器接口
│   ├── test_generator.rs# 核心测试生成逻辑
│   └── go_templates.rs  # Go测试代码模板
├── validator/           # 测试验证器
│   └── mod.rs           # 覆盖率验证和代码质量分析
└── types/               # 类型定义
    └── mod.rs           # 所有数据类型和错误定义
```

### 核心组件

#### 1. CoverAgent (主协调器)
- 管理整个测试生成流程
- 协调各个子组件的工作
- 提供统一的API接口

#### 2. TrpcAnalyzer (Go代码分析器)
- 解析tRPC路由定义
- 识别输入/输出类型
- 分析项目结构和依赖

#### 3. TestGenerator (测试生成器)
- 生成符合Go标准的测试代码
- 支持多种测试类型
- 使用模板化生成

#### 4. TestValidator (测试验证器)
- 验证生成的测试代码
- 分析测试覆盖率
- 提供代码质量报告

## 🚀 快速开始

### 安装

```bash
# 克隆项目
git clone <project-url>
cd tabby/crates/trpc-test-generator

# 构建项目
cargo build --release
```

### 基本使用

#### 1. 分析tRPC项目
```bash
# 分析项目结构
./target/release/trpc-test-gen analyze \
  --project-path ./my-trpc-project \
  --output analysis.json
```

#### 2. 生成测试用例
```bash
# 使用OpenAI生成测试
./target/release/trpc-test-gen generate \
  --project-path ./my-trpc-project \
  --output-dir ./generated_tests \
  --model gpt-4 \
  --api-key YOUR_API_KEY

# 使用本地Tabby生成测试
./target/release/trpc-test-gen generate \
  --project-path ./my-trpc-project \
  --output-dir ./generated_tests \
  --model tabby
```

#### 3. 验证测试覆盖率
```bash
# 验证生成的测试
./target/release/trpc-test-gen validate \
  --project-path ./my-trpc-project \
  --test-path ./generated_tests
```

### 高级配置

#### 配置文件示例 (trpc-test-config.toml)
```toml
[llm]
provider = "openai"  # openai, claude, tabby
model_name = "gpt-4"
api_key = "your-api-key"
timeout_seconds = 60
max_retries = 3

[generation]
generate_unit_tests = true
generate_integration_tests = true
generate_performance_tests = false
generate_mocks = true
target_coverage = 80.0
max_test_cases_per_route = 10
```

## 📖 使用示例

### 示例tRPC项目结构

```go
// handlers/user.go
package handlers

import "context"

type UserRequest struct {
    ID   int    `json:"id"`
    Name string `json:"name"`
}

type UserResponse struct {
    User    User `json:"user"`
    Success bool `json:"success"`
}

func GetUserHandler(ctx context.Context, req *UserRequest) (*UserResponse, error) {
    // 实现逻辑
    return &UserResponse{Success: true}, nil
}

// routes/routes.go
package routes

func SetupRoutes(r *Router) {
    r.Query("getUser", handlers.GetUserHandler)
    r.Mutation("updateUser", handlers.UpdateUserHandler)
}
```

### 生成的测试代码示例

```go
package handlers

import (
    "context"
    "testing"
    "github.com/stretchr/testify/assert"
)

// getUser 测试 getUser 路由的基本功能
func TestGetUserHandler(t *testing.T) {
    ctx := context.Background()

    // 准备测试数据
    testCases := []struct {
        name    string
        input   *UserRequest
        wantErr bool
    }{
        {
            name: "valid_request",
            input: &UserRequest{
                ID:   123,
                Name: "test_name",
            },
            wantErr: false,
        },
        {
            name:    "nil_request",
            input:   nil,
            wantErr: true,
        },
    }

    for _, tc := range testCases {
        t.Run(tc.name, func(t *testing.T) {
            result, err := GetUserHandler(ctx, tc.input)

            if tc.wantErr {
                assert.Error(t, err, "期望出现错误")
                assert.Nil(t, result, "出错时结果应为nil")
            } else {
                assert.NoError(t, err, "不应该出现错误")
                assert.NotNil(t, result, "成功时应返回结果")
            }
        })
    }
}
```

## 🛠️ 开发指南

### 添加新的LLM提供商

1. 在 `completion.rs` 中实现新的提供商结构
2. 实现 `LlmCompletionProvider` trait
3. 在 `create_llm_provider` 函数中添加新的提供商

```rust
pub struct NewLlmProvider {
    config: LlmConfig,
}

#[async_trait]
impl LlmCompletionProvider for NewLlmProvider {
    async fn generate_completion(&self, prompt: &str) -> Result<String, AgentError> {
        // 实现具体的LLM调用逻辑
    }
}
```

### 添加新的测试模板

1. 在 `go_templates.rs` 中添加新的模板函数
2. 在 `test_generator.rs` 中调用新模板
3. 确保模板符合Go测试标准

### 扩展代码分析能力

1. 在 `trpc_analyzer.rs` 中添加新的分析模式
2. 扩展正则表达式或使用AST解析
3. 更新类型定义以支持新的分析结果

## 🧪 测试

### 运行测试
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test agent::
cargo test generator::
cargo test validator::

# 运行集成测试
cargo test --test integration
```

### 测试覆盖率
```bash
# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir target/coverage
```

## 📊 性能和质量指标

### 支持的项目规模
- ✅ 小型项目 (< 10个路由)
- ✅ 中型项目 (10-100个路由)
- ✅ 大型项目 (> 100个路由)

### 测试质量评分标准
- **基础评分** (20分): 有测试函数
- **断言评分** (20分): 充分的断言覆盖
- **表驱动测试** (20分): 使用表驱动测试模式
- **错误处理** (20分): 完善的错误处理测试
- **Mock使用** (10分): 合理使用Mock
- **性能测试** (10分): 包含基准测试

### 目标覆盖率
- 🎯 **单元测试覆盖率**: > 80%
- 🎯 **集成测试覆盖率**: > 60%
- 🎯 **错误路径覆盖率**: > 70%

## 🔧 依赖项

### 核心依赖
- `rig-core`: LLM框架集成
- `tokio`: 异步运行时
- `serde`: 序列化支持
- `regex`: 正则表达式匹配
- `clap`: 命令行参数解析

### 可选依赖
- `tree-sitter-go`: Go语法树解析
- `tabby-inference`: 本地AI推理
- `reqwest`: HTTP客户端

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'Add amazing feature'`
4. 推送分支: `git push origin feature/amazing-feature`
5. 创建 Pull Request

### 代码规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 确保所有测试通过
- 添加适当的文档和注释

## 📝 许可证

本项目采用 Apache-2.0 许可证。详情请参见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- 感谢 [qodo-ai/qodo-cover](https://github.com/qodo-ai/qodo-cover) 项目的启发
- 感谢 [rig-dev](https://github.com/rig-dev/rig) 项目提供的LLM框架
- 感谢 Tabby 团队提供的代码智能基础设施

## 📞 支持与反馈

- 🐛 报告Bug: [GitHub Issues](https://github.com/your-org/trpc-test-generator/issues)
- 💡 功能建议: [GitHub Discussions](https://github.com/your-org/trpc-test-generator/discussions)
- 📧 联系我们: your-email@example.com

---

**让AI为你的tRPC项目生成高质量的测试代码！** 🚀