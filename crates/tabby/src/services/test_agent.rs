use std::sync::Arc;
use crate::services::completion::CompletionService;
use anyhow::Result;

#[derive(Clone)]
pub struct TestAgentService {
    completion_service: Arc<CompletionService>,
}

impl TestAgentService {
    pub fn new(completion_service: Arc<CompletionService>) -> Self {
        Self { completion_service }
    }

    pub async fn generate_test_case(&self, api_desc: &str) -> Result<String> {
        // 构造prompt
        let prompt = format!(
            "请为如下接口生成详细的测试用例，包含正向和异常场景。请用中文回答，并按照以下格式输出：\n\n\
            1. 测试场景概述\n\
            2. 测试用例列表（每个用例包含：用例名称、前置条件、测试步骤、预期结果）\n\
            3. 测试数据准备\n\
            4. 注意事项\n\n\
            接口描述：{}",
            api_desc
        );

        // 调用本地模型生成测试用例
        let response = self.completion_service
            .generate_completion(&prompt, None)
            .await?;

        Ok(response.text)
    }
}