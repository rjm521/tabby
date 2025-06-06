pub mod completion;
pub mod trpc_analyzer;

use async_trait::async_trait;
use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn, debug};

use crate::types::{
    TrpcSchema, GeneratedTests, GeneratedTestFile, AgentError,
    LlmConfig, TestGenerationConfig, TestScenario, TestType,
};
use completion::LlmCompletionProvider;
use trpc_analyzer::TrpcAnalyzer;

/// Agent完成接口 - 类似qodo-cover的AgentCompletionABC
#[async_trait]
pub trait AgentCompletion {
    /// 生成测试用例
    async fn generate_tests(
        &self,
        source_file: &str,
        existing_tests: &str,
        coverage_report: &str,
        trpc_schema: &TrpcSchema,
    ) -> Result<GeneratedTestFile, AgentError>;

    /// 分析tRPC路由
    async fn analyze_trpc_routes(&self, source_code: &str) -> Result<Vec<crate::types::TrpcRoute>, AgentError>;

    /// 分析项目结构
    async fn analyze_project_structure(&self, project_path: &PathBuf) -> Result<TrpcSchema, AgentError>;
}

/// CoverAgent - 主要协调器，管理整个测试生成流程
pub struct CoverAgent {
    /// LLM完成提供商
    llm_provider: Box<dyn LlmCompletionProvider>,
    /// tRPC分析器
    trpc_analyzer: TrpcAnalyzer,
    /// 测试生成配置
    generation_config: TestGenerationConfig,
}

impl CoverAgent {
    /// 创建新的CoverAgent实例
    pub async fn new(model_name: String, api_key: Option<String>) -> Result<Self, AgentError> {
        let llm_config = LlmConfig {
            model_name,
            api_key,
            ..Default::default()
        };

        let llm_provider = completion::create_llm_provider(llm_config).await?;
        let trpc_analyzer = TrpcAnalyzer::new();
        let generation_config = TestGenerationConfig::default();

        Ok(Self {
            llm_provider,
            trpc_analyzer,
            generation_config,
        })
    }

    /// 使用自定义配置创建CoverAgent
    pub async fn with_config(
        llm_config: LlmConfig,
        generation_config: TestGenerationConfig,
    ) -> Result<Self, AgentError> {
        let llm_provider = completion::create_llm_provider(llm_config).await?;
        let trpc_analyzer = TrpcAnalyzer::new();

        Ok(Self {
            llm_provider,
            trpc_analyzer,
            generation_config,
        })
    }
}

#[async_trait]
impl AgentCompletion for CoverAgent {
    /// 生成测试用例
    async fn generate_tests(
        &self,
        source_file: &str,
        existing_tests: &str,
        coverage_report: &str,
        trpc_schema: &TrpcSchema,
    ) -> Result<GeneratedTestFile, AgentError> {
        info!("开始生成测试用例");

        // 1. 分析源代码中的tRPC路由
        let routes = self.analyze_trpc_routes(source_file).await?;
        if routes.is_empty() {
            return Err(AgentError::CodeAnalysisError("未找到tRPC路由定义".to_string()));
        }

        let route = &routes[0]; // 假设我们处理第一个路由

        // 2. 构建测试生成的prompt
        let prompt = self.build_test_generation_prompt(
            route,
            source_file,
            existing_tests,
            coverage_report,
            trpc_schema,
        ).await?;

        // 3. 调用LLM生成测试代码
        debug!("调用LLM生成测试代码");
        let generated_code = self.llm_provider.generate_completion(&prompt).await?;

        // 4. 分析生成的测试代码，提取测试场景
        let scenarios = self.extract_test_scenarios(&generated_code)?;

        // 5. 构建返回结果
        let test_file = GeneratedTestFile {
            route_name: route.name.clone(),
            test_code: generated_code,
            test_case_count: scenarios.len(),
            scenarios,
        };

        info!("成功生成 {} 个测试用例", test_file.test_case_count);
        Ok(test_file)
    }

    /// 分析tRPC路由
    async fn analyze_trpc_routes(&self, source_code: &str) -> Result<Vec<crate::types::TrpcRoute>, AgentError> {
        debug!("分析tRPC路由");
        self.trpc_analyzer.analyze_routes(source_code).await
    }

    /// 分析项目结构
    async fn analyze_project_structure(&self, project_path: &PathBuf) -> Result<TrpcSchema, AgentError> {
        info!("分析项目结构: {:?}", project_path);
        self.trpc_analyzer.analyze_project(project_path).await
    }
}

impl CoverAgent {
    /// 构建测试生成的prompt
    async fn build_test_generation_prompt(
        &self,
        route: &crate::types::TrpcRoute,
        source_file: &str,
        existing_tests: &str,
        coverage_report: &str,
        trpc_schema: &TrpcSchema,
    ) -> Result<String, AgentError> {
        let mut prompt = String::new();

        prompt.push_str("你是一个专业的Go语言测试代码生成专家，专门为tRPC-Go项目生成高质量的测试用例。\n\n");

        // 添加路由信息
        prompt.push_str(&format!("## 待测试的tRPC路由信息\n"));
        prompt.push_str(&format!("- 路由名称: {}\n", route.name));
        prompt.push_str(&format!("- 路由类型: {:?}\n", route.route_type));
        prompt.push_str(&format!("- 处理函数: {}\n", route.handler_name));

        if let Some(input_type) = &route.input_type {
            prompt.push_str(&format!("- 输入类型: {}\n", input_type));
        }

        if let Some(output_type) = &route.output_type {
            prompt.push_str(&format!("- 输出类型: {}\n", output_type));
        }

        // 添加源代码
        prompt.push_str("\n## 源代码\n");
        prompt.push_str("```go\n");
        prompt.push_str(source_file);
        prompt.push_str("\n```\n\n");

        // 添加现有测试（如果有）
        if !existing_tests.is_empty() {
            prompt.push_str("## 现有测试代码\n");
            prompt.push_str("```go\n");
            prompt.push_str(existing_tests);
            prompt.push_str("\n```\n\n");
        }

        // 添加覆盖率报告（如果有）
        if !coverage_report.is_empty() {
            prompt.push_str("## 当前覆盖率报告\n");
            prompt.push_str(coverage_report);
            prompt.push_str("\n\n");
        }

        // 添加生成要求
        prompt.push_str("## 测试生成要求\n");
        prompt.push_str("请为上述tRPC路由生成完整的Go测试代码，包括：\n\n");

        if self.generation_config.generate_unit_tests {
            prompt.push_str("1. **单元测试**: 测试路由处理函数的核心逻辑\n");
        }

        if self.generation_config.generate_integration_tests {
            prompt.push_str("2. **集成测试**: 测试完整的tRPC调用流程\n");
        }

        prompt.push_str("3. **边界测试**: 测试输入参数的边界情况\n");
        prompt.push_str("4. **错误处理测试**: 测试各种错误情况的处理\n");

        if self.generation_config.generate_mocks {
            prompt.push_str("5. **Mock测试**: 生成必要的mock对象\n");
        }

        prompt.push_str("\n## 代码要求\n");
        prompt.push_str("- 使用Go标准测试框架（testing包）\n");
        prompt.push_str("- 遵循Go测试命名约定（Test*函数）\n");
        prompt.push_str("- 包含详细的测试注释\n");
        prompt.push_str("- 使用表驱动测试（table-driven tests）\n");
        prompt.push_str("- 确保测试代码可以直接运行\n");
        prompt.push_str("- 包含适当的断言和错误检查\n\n");

        prompt.push_str("请直接生成完整的Go测试代码，不需要额外的解释。");

        Ok(prompt)
    }

    /// 从生成的测试代码中提取测试场景
    fn extract_test_scenarios(&self, test_code: &str) -> Result<Vec<TestScenario>, AgentError> {
        let mut scenarios = Vec::new();

        // 使用正则表达式匹配测试函数
        let re = regex::Regex::new(r"func\s+(Test\w+)\s*\(.*?\)\s*\{").unwrap();

        for cap in re.captures_iter(test_code) {
            let test_func_name = cap.get(1).unwrap().as_str();

            let scenario = TestScenario {
                name: test_func_name.to_string(),
                description: format!("测试函数: {}", test_func_name),
                test_type: self.infer_test_type(test_func_name),
                expected_result: "测试通过".to_string(),
            };

            scenarios.push(scenario);
        }

        // 如果没有找到测试函数，添加默认场景
        if scenarios.is_empty() {
            scenarios.push(TestScenario {
                name: "DefaultTest".to_string(),
                description: "默认测试场景".to_string(),
                test_type: TestType::Unit,
                expected_result: "测试通过".to_string(),
            });
        }

        Ok(scenarios)
    }

    /// 根据测试函数名推断测试类型
    fn infer_test_type(&self, test_func_name: &str) -> TestType {
        let name_lower = test_func_name.to_lowercase();

        if name_lower.contains("integration") {
            TestType::Integration
        } else if name_lower.contains("error") || name_lower.contains("fail") {
            TestType::ErrorHandling
        } else if name_lower.contains("boundary") || name_lower.contains("edge") {
            TestType::Boundary
        } else if name_lower.contains("performance") || name_lower.contains("benchmark") {
            TestType::Performance
        } else {
            TestType::Unit
        }
    }
}