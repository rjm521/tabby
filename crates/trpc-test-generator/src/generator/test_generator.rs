use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::generator::{TestCodeGenerator, TestGenerationContext};
use crate::generator::go_templates::GoTestTemplate;
use crate::types::{
    AgentError, GeneratedTestFile, TestGenerationConfig, TestScenario, TestType, TrpcField,
    TrpcRoute, TrpcRouteType, TrpcSchema, TrpcType,
};

/// 测试生成器 - 负责生成tRPC-Go测试代码
pub struct TestGenerator {
    /// Go测试模板
    template: GoTestTemplate,
}

impl TestGenerator {
    /// 创建新的测试生成器
    pub fn new() -> Self {
        Self {
            template: GoTestTemplate::new(),
        }
    }

    /// 为路由生成完整的测试文件
    pub fn generate_test_file(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
        config: &TestGenerationConfig,
    ) -> Result<GeneratedTestFile, AgentError> {
        info!("为路由 {} 生成测试文件", route.name);

        let mut test_scenarios = Vec::new();
        let mut test_code_parts = Vec::new();

        // 生成包声明和导入
        test_code_parts.push(self.generate_package_and_imports(schema)?);

        // 生成基础测试
        if config.generate_unit_tests {
            let (unit_test, scenarios) = self.generate_unit_tests(route, schema)?;
            test_code_parts.push(unit_test);
            test_scenarios.extend(scenarios);
        }

        // 生成集成测试
        if config.generate_integration_tests {
            let (integration_test, scenarios) = self.generate_integration_tests(route, schema)?;
            test_code_parts.push(integration_test);
            test_scenarios.extend(scenarios);
        }

        // 生成边界测试
        let (boundary_test, scenarios) = self.generate_boundary_tests(route, schema)?;
        test_code_parts.push(boundary_test);
        test_scenarios.extend(scenarios);

        // 生成错误处理测试
        let (error_test, scenarios) = self.generate_error_tests(route, schema)?;
        test_code_parts.push(error_test);
        test_scenarios.extend(scenarios);

        // 生成Mock代码（如果需要）
        if config.generate_mocks {
            let mock_code = self.generate_mock_code(route, schema)?;
            test_code_parts.push(mock_code);
        }

        // 生成性能测试（如果需要）
        if config.generate_performance_tests {
            let (perf_test, scenarios) = self.generate_performance_tests(route, schema)?;
            test_code_parts.push(perf_test);
            test_scenarios.extend(scenarios);
        }

        let final_test_code = test_code_parts.join("\n\n");

        Ok(GeneratedTestFile {
            route_name: route.name.clone(),
            test_code: final_test_code,
            test_case_count: test_scenarios.len(),
            scenarios: test_scenarios,
        })
    }

    /// 生成包声明和导入语句
    fn generate_package_and_imports(&self, schema: &TrpcSchema) -> Result<String, AgentError> {
        let mut code = String::new();

        // 包声明
        code.push_str(&format!("package {}\n\n", schema.config.package_name));

        // 导入语句
        code.push_str("import (\n");
        code.push_str("    \"context\"\n");
        code.push_str("    \"testing\"\n");
        code.push_str("    \"time\"\n");
        code.push_str("    \"errors\"\n");
        code.push_str("    \"fmt\"\n");
        code.push_str("    \"encoding/json\"\n");
        code.push_str("    \"github.com/stretchr/testify/assert\"\n");
        code.push_str("    \"github.com/stretchr/testify/require\"\n");
        code.push_str("    \"github.com/stretchr/testify/mock\"\n");

        // 添加项目特定的导入
        for import_path in &schema.config.import_paths {
            if !import_path.starts_with("github.com/stretchr/testify")
                && !["context", "testing", "time", "errors", "fmt", "encoding/json"].contains(&import_path.as_str()) {
                code.push_str(&format!("    \"{}\"\n", import_path));
            }
        }

        code.push_str(")\n");

        Ok(code)
    }

    /// 生成单元测试
    fn generate_unit_tests(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
    ) -> Result<(String, Vec<TestScenario>), AgentError> {
        debug!("生成单元测试: {}", route.name);

        let mut scenarios = Vec::new();
        let func_name = format!("Test{}", self.capitalize(&route.handler_name));

        // 获取输入输出类型
        let input_type = self.find_type_by_name(schema, &route.input_type);
        let output_type = self.find_type_by_name(schema, &route.output_type);

        // 生成测试数据
        let test_data = if let Some(input) = &input_type {
            self.generate_test_data(input)?
        } else {
            "nil".to_string()
        };

        let expected_output = if let Some(output) = &output_type {
            self.generate_test_data(output)?
        } else {
            "nil".to_string()
        };

        let test_code = self.template.render_unit_test(&HashMap::from([
            ("FuncName".to_string(), func_name.clone()),
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
            ("TestData".to_string(), test_data),
            ("ExpectedOutput".to_string(), expected_output),
            ("RouteType".to_string(), format!("{:?}", route.route_type)),
        ]))?;

        scenarios.push(TestScenario {
            name: func_name,
            description: format!("测试 {} 路由的基本功能", route.name),
            test_type: TestType::Unit,
            expected_result: "成功处理请求并返回预期结果".to_string(),
        });

        Ok((test_code, scenarios))
    }

    /// 生成集成测试
    fn generate_integration_tests(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
    ) -> Result<(String, Vec<TestScenario>), AgentError> {
        debug!("生成集成测试: {}", route.name);

        let mut scenarios = Vec::new();
        let func_name = format!("Test{}_Integration", self.capitalize(&route.handler_name));

        let test_code = self.template.render_integration_test(&HashMap::from([
            ("FuncName".to_string(), func_name.clone()),
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
            ("RouteType".to_string(), format!("{:?}", route.route_type)),
        ]))?;

        scenarios.push(TestScenario {
            name: func_name,
            description: format!("测试 {} 路由的完整tRPC调用流程", route.name),
            test_type: TestType::Integration,
            expected_result: "完整的tRPC调用流程正常工作".to_string(),
        });

        Ok((test_code, scenarios))
    }

    /// 生成边界测试
    fn generate_boundary_tests(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
    ) -> Result<(String, Vec<TestScenario>), AgentError> {
        debug!("生成边界测试: {}", route.name);

        let mut scenarios = Vec::new();
        let func_name = format!("Test{}_Boundary", self.capitalize(&route.handler_name));

        // 基于输入类型生成边界测试用例
        let boundary_cases = if let Some(input_type) = self.find_type_by_name(schema, &route.input_type) {
            self.generate_boundary_test_cases(input_type)?
        } else {
            vec![("empty_input".to_string(), "nil".to_string())]
        };

        let test_code = self.template.render_boundary_test(&HashMap::from([
            ("FuncName".to_string(), func_name.clone()),
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
            ("BoundaryCases".to_string(), serde_json::to_string(&boundary_cases).unwrap()),
        ]))?;

        scenarios.push(TestScenario {
            name: func_name,
            description: format!("测试 {} 路由的边界条件", route.name),
            test_type: TestType::Boundary,
            expected_result: "正确处理各种边界情况".to_string(),
        });

        Ok((test_code, scenarios))
    }

    /// 生成错误处理测试
    fn generate_error_tests(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
    ) -> Result<(String, Vec<TestScenario>), AgentError> {
        debug!("生成错误处理测试: {}", route.name);

        let mut scenarios = Vec::new();
        let func_name = format!("Test{}_Error", self.capitalize(&route.handler_name));

        let test_code = self.template.render_error_test(&HashMap::from([
            ("FuncName".to_string(), func_name.clone()),
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
        ]))?;

        scenarios.push(TestScenario {
            name: func_name,
            description: format!("测试 {} 路由的错误处理", route.name),
            test_type: TestType::ErrorHandling,
            expected_result: "正确处理和返回各种错误情况".to_string(),
        });

        Ok((test_code, scenarios))
    }

    /// 生成性能测试
    fn generate_performance_tests(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
    ) -> Result<(String, Vec<TestScenario>), AgentError> {
        debug!("生成性能测试: {}", route.name);

        let mut scenarios = Vec::new();
        let func_name = format!("Benchmark{}", self.capitalize(&route.handler_name));

        let test_code = self.template.render_benchmark_test(&HashMap::from([
            ("FuncName".to_string(), func_name.clone()),
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
        ]))?;

        scenarios.push(TestScenario {
            name: func_name,
            description: format!("测试 {} 路由的性能", route.name),
            test_type: TestType::Performance,
            expected_result: "路由处理性能在可接受范围内".to_string(),
        });

        Ok((test_code, scenarios))
    }

    /// 生成边界测试用例
    fn generate_boundary_test_cases(&self, input_type: &TrpcType) -> Result<Vec<(String, String)>, AgentError> {
        let mut cases = Vec::new();

        for field in &input_type.fields {
            match field.field_type.as_str() {
                "string" => {
                    cases.push(("empty_string".to_string(), format!("\"\""))); // 空字符串
                    cases.push(("long_string".to_string(), format!("\"{}\"", "a".repeat(1000)))); // 长字符串
                }
                "int" | "int32" | "int64" => {
                    cases.push(("zero".to_string(), "0".to_string()));
                    cases.push(("negative".to_string(), "-1".to_string()));
                    cases.push(("max_value".to_string(), "2147483647".to_string()));
                }
                "[]string" => {
                    cases.push(("empty_slice".to_string(), "[]string{}".to_string()));
                    cases.push(("nil_slice".to_string(), "nil".to_string()));
                }
                _ => {
                    cases.push(("nil_value".to_string(), "nil".to_string()));
                }
            }
        }

        // 如果没有找到任何字段，添加一个默认的边界测试
        if cases.is_empty() {
            cases.push(("nil_input".to_string(), "nil".to_string()));
        }

        Ok(cases)
    }

    /// 根据名称查找类型定义
    fn find_type_by_name<'a>(&self, schema: &'a TrpcSchema, type_name: &Option<String>) -> Option<&'a TrpcType> {
        if let Some(name) = type_name {
            schema.types.iter().find(|t| &t.name == name)
        } else {
            None
        }
    }

    /// 首字母大写
    fn capitalize(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

impl TestCodeGenerator for TestGenerator {
    /// 为单个路由生成测试代码
    fn generate_route_test(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
        config: &TestGenerationConfig,
    ) -> Result<String, AgentError> {
        info!("为路由 {} 生成测试代码", route.name);

        let mut test_code = String::new();

        // 生成包声明和导入
        test_code.push_str(&self.generate_package_and_imports(schema)?);
        test_code.push_str("\n\n");

        // 生成基础测试
        if config.generate_unit_tests {
            test_code.push_str(&self.generate_unit_test(route, schema)?);
            test_code.push_str("\n\n");
        }

        // 生成错误处理测试
        test_code.push_str(&self.generate_error_test(route, schema)?);
        test_code.push_str("\n\n");

        // 生成Mock代码（如果需要）
        if config.generate_mocks {
            test_code.push_str(&self.generate_mock_code(route, schema)?);
        }

        Ok(test_code)
    }

    /// 生成Mock代码
    fn generate_mock_code(
        &self,
        route: &TrpcRoute,
        schema: &TrpcSchema,
    ) -> Result<String, AgentError> {
        debug!("生成Mock代码: {}", route.name);
        self.template.render_mock_code(&HashMap::from([
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
        ]))
    }

    /// 生成测试数据
    fn generate_test_data(&self, trpc_type: &TrpcType) -> Result<String, AgentError> {
        debug!("生成测试数据: {}", trpc_type.name);

        let mut data_parts = Vec::new();
        data_parts.push(format!("&{} {{", trpc_type.name));

        for field in &trpc_type.fields {
            let field_value = match field.field_type.as_str() {
                "string" => format!("\"test_{}\"", field.name),
                "int" | "int32" | "int64" => "123".to_string(),
                "bool" => "true".to_string(),
                _ => "nil".to_string(),
            };
            data_parts.push(format!("    {}: {},", field.name, field_value));
        }

        data_parts.push("}".to_string());
        Ok(data_parts.join("\n"))
    }
}

impl TestGenerator {

    /// 生成单元测试
    fn generate_unit_test(&self, route: &TrpcRoute, schema: &TrpcSchema) -> Result<String, AgentError> {
        let func_name = format!("Test{}", self.capitalize(&route.handler_name));
        self.template.render_unit_test(&HashMap::from([
            ("FuncName".to_string(), func_name),
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
        ]))
    }

    /// 生成错误处理测试
    fn generate_error_test(&self, route: &TrpcRoute, schema: &TrpcSchema) -> Result<String, AgentError> {
        let func_name = format!("Test{}_Error", self.capitalize(&route.handler_name));
        self.template.render_error_test(&HashMap::from([
            ("FuncName".to_string(), func_name),
            ("HandlerName".to_string(), route.handler_name.clone()),
            ("RouteName".to_string(), route.name.clone()),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TrpcConfig, TrpcRouteType};

    fn create_test_schema() -> TrpcSchema {
        TrpcSchema {
            project_name: "test_project".to_string(),
            routes: Vec::new(),
            types: vec![
                TrpcType {
                    name: "UserRequest".to_string(),
                    go_type: "struct UserRequest".to_string(),
                    fields: vec![
                        TrpcField {
                            name: "id".to_string(),
                            field_type: "int".to_string(),
                            required: true,
                            validation: None,
                            description: None,
                        },
                        TrpcField {
                            name: "name".to_string(),
                            field_type: "string".to_string(),
                            required: true,
                            validation: None,
                            description: None,
                        },
                    ],
                    is_input: true,
                    is_output: false,
                },
            ],
            config: TrpcConfig {
                package_name: "main".to_string(),
                import_paths: vec!["context".to_string()],
                global_middlewares: Vec::new(),
            },
        }
    }

    #[test]
    fn test_generate_test_data() {
        let generator = TestGenerator::new();
        let schema = create_test_schema();
        let user_type = &schema.types[0];

        let test_data = generator.generate_test_data(user_type).unwrap();
        assert!(test_data.contains("&UserRequest"));
        assert!(test_data.contains("id: 123"));
        assert!(test_data.contains("name: \"test_name\""));
    }

    #[test]
    fn test_generate_boundary_test_cases() {
        let generator = TestGenerator::new();
        let schema = create_test_schema();
        let user_type = &schema.types[0];

        let cases = generator.generate_boundary_test_cases(user_type).unwrap();
        assert!(!cases.is_empty());

        // 应该包含字符串和整数的边界测试用例
        assert!(cases.iter().any(|(name, _)| name.contains("string")));
        assert!(cases.iter().any(|(name, _)| name.contains("zero") || name.contains("negative")));
    }
}