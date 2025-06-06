use async_trait::async_trait;
use anyhow::Result;
// TODO: 添加rig框架依赖后启用
// use rig::{
//     completion::{Completion, CompletionModel, Prompt},
//     providers::{openai, anthropic},
// };
use serde_json::Value;
use std::time::Duration;
use tracing::{info, debug, error, warn};

use crate::types::{AgentError, LlmConfig};

/// LLM完成提供商trait
#[async_trait]
pub trait LlmCompletionProvider: Send + Sync {
    /// 生成文本完成
    async fn generate_completion(&self, prompt: &str) -> Result<String, AgentError>;

    /// 获取模型信息
    fn get_model_info(&self) -> &str;
}

/// OpenAI提供商实现 (TODO: 添加rig依赖后启用)
pub struct OpenAiProvider {
    // model: Box<dyn CompletionModel>,
    config: LlmConfig,
}

impl OpenAiProvider {
    pub async fn new(config: LlmConfig) -> Result<Self, AgentError> {
        let _api_key = config.api_key.clone()
            .ok_or_else(|| AgentError::ConfigError("OpenAI API key is required".to_string()))?;

        // TODO: 启用rig框架后恢复
        // 创建OpenAI客户端
        // let client = openai::Client::new(&api_key);

        info!("初始化OpenAI提供商，模型: {}", config.model_name);
        Ok(Self { config })
    }
}

#[async_trait]
impl LlmCompletionProvider for OpenAiProvider {
    async fn generate_completion(&self, _prompt: &str) -> Result<String, AgentError> {
        debug!("发送请求到OpenAI API (TODO: 实现)");

        // TODO: 启用rig框架后实现
        // let completion_request = Prompt::from_template(prompt);

        // 临时返回示例代码
        warn!("OpenAI提供商暂未实现，返回示例测试代码");
        Ok(format!(r#"
func TestExampleHandler(t *testing.T) {{
    ctx := context.Background()

    testCases := []struct {{
        name    string
        input   interface{{}}
        wantErr bool
    }}{{
        {{
            name:    "valid_request",
            input:   nil,
            wantErr: false,
        }},
    }}

    for _, tc := range testCases {{
        t.Run(tc.name, func(t *testing.T) {{
            result, err := ExampleHandler(ctx, tc.input)

            if tc.wantErr {{
                assert.Error(t, err)
            }} else {{
                assert.NoError(t, err)
                assert.NotNil(t, result)
            }}
        }})
    }}
}}"#))
    }

    fn get_model_info(&self) -> &str {
        &self.config.model_name
    }
}

/// Claude提供商实现 (TODO: 添加rig依赖后启用)
pub struct ClaudeProvider {
    // model: Box<dyn CompletionModel>,
    config: LlmConfig,
}

impl ClaudeProvider {
    pub async fn new(config: LlmConfig) -> Result<Self, AgentError> {
        let _api_key = config.api_key.clone()
            .ok_or_else(|| AgentError::ConfigError("Claude API key is required".to_string()))?;

        // TODO: 启用rig框架后恢复
        // 创建Anthropic客户端
        // let client = anthropic::Client::new(&api_key);

        info!("初始化Claude提供商，模型: {}", config.model_name);
        Ok(Self { config })
    }
}

#[async_trait]
impl LlmCompletionProvider for ClaudeProvider {
    async fn generate_completion(&self, _prompt: &str) -> Result<String, AgentError> {
        debug!("发送请求到Claude API (TODO: 实现)");

        // TODO: 启用rig框架后实现
        // let completion_request = Prompt::from_template(prompt);

        // 临时返回示例代码
        warn!("Claude提供商暂未实现，返回示例测试代码");
        Ok(format!(r#"
func TestExampleHandler(t *testing.T) {{
    ctx := context.Background()

    // Claude生成的示例测试
    testCases := []struct {{
        name    string
        input   interface{{}}
        wantErr bool
    }}{{
        {{
            name:    "valid_request",
            input:   nil,
            wantErr: false,
        }},
        {{
            name:    "invalid_request",
            input:   "invalid",
            wantErr: true,
        }},
    }}

    for _, tc := range testCases {{
        t.Run(tc.name, func(t *testing.T) {{
            result, err := ExampleHandler(ctx, tc.input)

            if tc.wantErr {{
                assert.Error(t, err)
            }} else {{
                assert.NoError(t, err)
                assert.NotNil(t, result)
            }}
        }})
    }}
}}"#))
    }

    fn get_model_info(&self) -> &str {
        &self.config.model_name
    }
}

/// 本地/Tabby提供商实现
pub struct TabbyProvider {
    client: reqwest::Client,
    config: LlmConfig,
}

impl TabbyProvider {
    pub async fn new(config: LlmConfig) -> Result<Self, AgentError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| AgentError::ConfigError(format!("创建HTTP客户端失败: {}", e)))?;

        info!("初始化Tabby本地提供商，模型: {}", config.model_name);
        Ok(Self { client, config })
    }
}

#[async_trait]
impl LlmCompletionProvider for TabbyProvider {
    async fn generate_completion(&self, prompt: &str) -> Result<String, AgentError> {
        debug!("发送请求到Tabby本地API");

        let default_url = "http://localhost:8080".to_string();
        let api_url = self.config.api_url.as_ref()
            .unwrap_or(&default_url);

        let request_body = serde_json::json!({
            "language": "go",
            "segments": {
                "prefix": prompt
            }
        });

        // 设置重试逻辑
        let mut attempts = 0;
        let max_attempts = self.config.max_retries;

        while attempts < max_attempts {
            let response = self.client
                .post(&format!("{}/v1/completions", api_url))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.text().await {
                            Ok(text) => {
                                debug!("成功收到Tabby响应");

                                // 解析Tabby响应格式
                                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                                        if let Some(first_choice) = choices.get(0) {
                                            if let Some(text) = first_choice.get("text").and_then(|t| t.as_str()) {
                                                return Ok(text.to_string());
                                            }
                                        }
                                    }
                                }

                                // 如果解析失败，返回原始文本
                                return Ok(text);
                            }
                            Err(e) => {
                                attempts += 1;
                                error!("解析Tabby响应失败 (尝试 {}/{}): {:?}", attempts, max_attempts, e);
                            }
                        }
                    } else {
                        attempts += 1;
                        error!("Tabby API返回错误状态 (尝试 {}/{}): {}", attempts, max_attempts, resp.status());
                    }
                }
                Err(e) => {
                    attempts += 1;
                    error!("Tabby API请求失败 (尝试 {}/{}): {:?}", attempts, max_attempts, e);
                }
            }

            if attempts >= max_attempts {
                return Err(AgentError::LlmApiError("Tabby API请求失败".to_string()));
            }

            // 指数退避
            let delay = Duration::from_secs(2_u64.pow(attempts as u32));
            tokio::time::sleep(delay).await;
        }

        Err(AgentError::LlmApiError("最大重试次数已达到".to_string()))
    }

    fn get_model_info(&self) -> &str {
        &self.config.model_name
    }
}

/// 创建LLM提供商实例
pub async fn create_llm_provider(config: LlmConfig) -> Result<Box<dyn LlmCompletionProvider>, AgentError> {
    match config.provider.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(config).await?;
            Ok(Box::new(provider))
        }
        "claude" | "anthropic" => {
            let provider = ClaudeProvider::new(config).await?;
            Ok(Box::new(provider))
        }
        "tabby" | "local" => {
            let provider = TabbyProvider::new(config).await?;
            Ok(Box::new(provider))
        }
        _ => Err(AgentError::ConfigError(format!("不支持的LLM提供商: {}", config.provider))),
    }
}

/// 创建用于代码分析的专用提供商
pub async fn create_code_analysis_provider() -> Result<Box<dyn LlmCompletionProvider>, AgentError> {
    // 优先使用本地Tabby，因为它专门针对代码优化
    let config = LlmConfig {
        provider: "tabby".to_string(),
        model_name: "CodeLlama-7B".to_string(),
        api_url: Some("http://localhost:8080".to_string()),
        ..Default::default()
    };

    create_llm_provider(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tabby_provider_creation() {
        let config = LlmConfig {
            provider: "tabby".to_string(),
            model_name: "test-model".to_string(),
            api_url: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };

        let provider = TabbyProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_create_llm_provider() {
        let config = LlmConfig {
            provider: "tabby".to_string(),
            model_name: "test-model".to_string(),
            ..Default::default()
        };

        let provider = create_llm_provider(config).await;
        assert!(provider.is_ok());
    }
}