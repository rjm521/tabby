pub mod test_generator;
pub mod go_templates;

use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::types::{
    AgentError, GeneratedTestFile, TestGenerationConfig, TestScenario, TestType, TrpcRoute,
    TrpcSchema, TrpcType,
};

pub use test_generator::TestGenerator;
pub use go_templates::GoTestTemplate;

/// 测试生成器接口
pub trait TestCodeGenerator {
    /// 为单个路由生成测试代码
    fn generate_route_test(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
        config: &TestGenerationConfig,
    ) -> Result<String, AgentError>;

    /// 生成Mock代码
    fn generate_mock_code(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
    ) -> Result<String, AgentError>;

    /// 生成测试数据
    fn generate_test_data(
        &self,
        trpc_type: &TrpcType,
    ) -> Result<String, AgentError>;
}

/// 测试代码生成上下文
#[derive(Debug, Clone)]
pub struct TestGenerationContext {
    /// 当前路由
    pub route: TrpcRoute,
    /// 项目schema
    pub schema: TrpcSchema,
    /// 生成配置
    pub config: TestGenerationConfig,
    /// 额外的上下文变量
    pub variables: HashMap<String, String>,
}

impl TestGenerationContext {
    pub fn new(
        route: TrpcRoute,
        schema: TrpcSchema,
        config: TestGenerationConfig,
    ) -> Self {
        Self {
            route,
            schema,
            config,
            variables: HashMap::new(),
        }
    }

    /// 添加上下文变量
    pub fn add_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// 获取上下文变量
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }
}