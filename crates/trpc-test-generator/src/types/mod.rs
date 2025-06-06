use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// tRPC项目的schema信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrpcSchema {
    /// 项目名称
    pub project_name: String,
    /// 所有tRPC路由
    pub routes: Vec<TrpcRoute>,
    /// 类型定义
    pub types: Vec<TrpcType>,
    /// 全局配置
    pub config: TrpcConfig,
}

/// tRPC路由定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrpcRoute {
    /// 路由名称
    pub name: String,
    /// 路由类型 (query, mutation, subscription)
    pub route_type: TrpcRouteType,
    /// 输入类型
    pub input_type: Option<String>,
    /// 输出类型
    pub output_type: Option<String>,
    /// 处理函数名称
    pub handler_name: String,
    /// 源文件路径
    pub source_file: PathBuf,
    /// 代码行号范围
    pub line_range: (usize, usize),
    /// 中间件列表
    pub middlewares: Vec<String>,
    /// 注释/文档
    pub documentation: Option<String>,
}

/// tRPC路由类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrpcRouteType {
    Query,
    Mutation,
    Subscription,
}

/// tRPC类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrpcType {
    /// 类型名称
    pub name: String,
    /// Go类型定义
    pub go_type: String,
    /// 字段定义
    pub fields: Vec<TrpcField>,
    /// 是否为输入类型
    pub is_input: bool,
    /// 是否为输出类型
    pub is_output: bool,
}

/// tRPC字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrpcField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub field_type: String,
    /// 是否必填
    pub required: bool,
    /// 验证规则
    pub validation: Option<String>,
    /// 字段注释
    pub description: Option<String>,
}

/// tRPC配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrpcConfig {
    /// 包名
    pub package_name: String,
    /// 导入路径
    pub import_paths: Vec<String>,
    /// 全局中间件
    pub global_middlewares: Vec<String>,
}

/// 生成的测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTests {
    /// 生成的测试文件
    pub test_files: Vec<GeneratedTestFile>,
    /// 总路由数量
    pub total_routes: usize,
    /// 覆盖率信息
    pub coverage_info: Option<CoverageResult>,
}

/// 单个测试文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestFile {
    /// 路由名称
    pub route_name: String,
    /// 测试代码
    pub test_code: String,
    /// 测试用例数量
    pub test_case_count: usize,
    /// 涵盖的场景
    pub scenarios: Vec<TestScenario>,
}

/// 测试场景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    /// 场景名称
    pub name: String,
    /// 场景描述
    pub description: String,
    /// 测试类型
    pub test_type: TestType,
    /// 期望结果
    pub expected_result: String,
}

/// 测试类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    /// 单元测试
    Unit,
    /// 集成测试
    Integration,
    /// 边界测试
    Boundary,
    /// 错误处理测试
    ErrorHandling,
    /// 性能测试
    Performance,
}

/// 覆盖率结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageResult {
    /// 覆盖率百分比
    pub coverage_percentage: f64,
    /// 已覆盖的行数
    pub covered_lines: usize,
    /// 总行数
    pub total_lines: usize,
    /// 未覆盖的函数
    pub uncovered_functions: Vec<String>,
    /// 覆盖率报告文件路径
    pub report_file: Option<PathBuf>,
}

/// Agent错误类型
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("LLM API错误: {0}")]
    LlmApiError(String),

    #[error("代码分析错误: {0}")]
    CodeAnalysisError(String),

    #[error("文件IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("HTTP请求错误: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("测试验证错误: {0}")]
    ValidationError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),
}

/// LLM提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 提供商名称 (openai, claude, local)
    pub provider: String,
    /// API密钥
    pub api_key: Option<String>,
    /// 模型名称
    pub model_name: String,
    /// API URL (用于本地部署)
    pub api_url: Option<String>,
    /// 请求超时时间(秒)
    pub timeout_seconds: u64,
    /// 最大重试次数
    pub max_retries: u32,
}

/// 测试生成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerationConfig {
    /// 是否生成单元测试
    pub generate_unit_tests: bool,
    /// 是否生成集成测试
    pub generate_integration_tests: bool,
    /// 是否生成性能测试
    pub generate_performance_tests: bool,
    /// 是否生成mock代码
    pub generate_mocks: bool,
    /// 测试覆盖率目标百分比
    pub target_coverage: f64,
    /// 生成的测试用例数量限制
    pub max_test_cases_per_route: usize,
}

impl Default for TestGenerationConfig {
    fn default() -> Self {
        Self {
            generate_unit_tests: true,
            generate_integration_tests: true,
            generate_performance_tests: false,
            generate_mocks: true,
            target_coverage: 80.0,
            max_test_cases_per_route: 10,
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: None,
            model_name: "gpt-3.5-turbo".to_string(),
            api_url: None,
            timeout_seconds: 60,
            max_retries: 3,
        }
    }
}