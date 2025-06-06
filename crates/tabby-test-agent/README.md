# tabby-test-agent

## 项目简介

`tabby-test-agent` 是一个基于 Rust 和 Rig 框架的智能测试用例生成服务。它可以根据你输入的接口描述，自动调用大模型（如 GPT-4）生成高质量的接口测试用例，帮助你快速提升接口的健壮性和开发效率。

---

## 主要功能

- 提供 HTTP API（/v1/test/generate），输入接口描述，自动生成测试用例
- 支持正向、异常、边界等多种测试场景
- 生成的测试用例可直接用于接口自动化测试
- 代码结构清晰，注释详细，方便理解和二次开发

---

## 使用方法

### 1. 环境准备
- 需要 Rust 1.70 及以上版本
- 需要在 `crates/tabby-test-agent/config.toml` 文件中配置 OpenAI API Key（或其他 Rig 支持的 LLM 提供商）
- 推荐使用 Linux 或 MacOS 环境，Windows 也可运行

### 2. 配置文件示例
```toml
[openai]
api_key = "你的key"
```

### 3. 安装依赖
```bash
cargo build
```

### 4. 启动服务
```bash
cargo run -p tabby-test-agent
```

### 5. 调用接口
- POST http://localhost:8080/v1/test/generate
- 请求体：
```json
{
  "api_desc": "接口描述，如：POST /v1/chat/completions，参数xxx，返回xxx"
}
```
- 返回示例：
```json
{
  "test_case": "这里是AI自动生成的测试用例代码或用例描述"
}
```

---

## 未来扩展建议
- 支持多种测试用例输出格式（如 Rust、Python、Postman 等）
- 支持自动解析 OpenAPI/Swagger 文档批量生成测试用例
- 集成更复杂的 Agent 编排能力，实现多步推理和智能测试
- 支持更多 LLM 后端（如本地模型、Cohere、Together 等）

---

## 常见问题
- 启动报错请检查 config.toml 是否存在且 api_key 是否配置正确
- 生成效果依赖于大模型能力和 prompt 设计，可根据实际需求优化 prompt

---

## 联系与反馈
如有问题或建议，请在项目 issue 区留言，开发者会第一时间响应。