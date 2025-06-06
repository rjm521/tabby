use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info, warn};

use crate::types::{AgentError, CoverageResult, GeneratedTests};

/// TestValidator - 验证生成的测试并检查覆盖率
pub struct TestValidator {
    /// Go命令路径
    go_command: String,
    /// 覆盖率分析模式
    coverage_mode: CoverageMode,
}

/// 覆盖率分析模式
#[derive(Debug, Clone)]
pub enum CoverageMode {
    /// 行覆盖率
    Line,
    /// 块覆盖率
    Block,
    /// 函数覆盖率
    Function,
}

impl TestValidator {
    /// 创建新的测试验证器
    pub fn new() -> Self {
        Self {
            go_command: "go".to_string(),
            coverage_mode: CoverageMode::Line,
        }
    }

    /// 使用自定义配置创建验证器
    pub fn with_config(go_command: String, coverage_mode: CoverageMode) -> Self {
        Self {
            go_command,
            coverage_mode,
        }
    }

    /// 验证生成的测试覆盖率
    pub async fn validate_coverage(
        &self,
        project_path: &PathBuf,
        generated_tests: &GeneratedTests,
    ) -> Result<CoverageResult, AgentError> {
        info!("开始验证测试覆盖率");

        // 1. 编译测试
        self.compile_tests(project_path).await?;

        // 2. 运行测试并生成覆盖率报告
        let coverage_file = self.run_tests_with_coverage(project_path).await?;

        // 3. 解析覆盖率报告
        let coverage_result = self.parse_coverage_report(project_path, &coverage_file).await?;

        // 4. 分析未覆盖的代码
        let enhanced_result = self.analyze_uncovered_code(project_path, coverage_result).await?;

        info!(
            "覆盖率验证完成: {:.2}%",
            enhanced_result.coverage_percentage
        );

        Ok(enhanced_result)
    }

    /// 验证单个测试文件
    pub async fn validate_single_test(
        &self,
        project_path: &PathBuf,
        test_file_path: &PathBuf,
    ) -> Result<bool, AgentError> {
        info!("验证单个测试文件: {:?}", test_file_path);

        // 检查测试文件是否存在
        if !test_file_path.exists() {
            return Err(AgentError::ValidationError(format!(
                "测试文件不存在: {:?}",
                test_file_path
            )));
        }

        // 运行go test检查语法
        let result = self.run_go_test_syntax_check(project_path, test_file_path).await?;

        info!("测试文件验证结果: {}", if result { "通过" } else { "失败" });
        Ok(result)
    }

    /// 检查测试代码质量
    pub async fn analyze_test_quality(
        &self,
        test_code: &str,
    ) -> Result<TestQualityReport, AgentError> {
        debug!("分析测试代码质量");

        let mut report = TestQualityReport::new();

        // 1. 检查测试函数数量
        report.test_function_count = self.count_test_functions(test_code);

        // 2. 检查断言数量
        report.assertion_count = self.count_assertions(test_code);

        // 3. 检查表驱动测试
        report.has_table_driven_tests = self.has_table_driven_tests(test_code);

        // 4. 检查错误处理测试
        report.has_error_handling_tests = self.has_error_handling_tests(test_code);

        // 5. 检查Mock使用
        report.uses_mocks = self.uses_mocks(test_code);

        // 6. 检查基准测试
        report.has_benchmark_tests = self.has_benchmark_tests(test_code);

        // 7. 计算质量评分
        report.quality_score = self.calculate_quality_score(&report);

        debug!("测试质量分析完成，评分: {:.2}", report.quality_score);
        Ok(report)
    }

    /// 编译测试
    async fn compile_tests(&self, project_path: &PathBuf) -> Result<(), AgentError> {
        debug!("编译测试...");

        let output = Command::new(&self.go_command)
            .args(&["test", "-c", "./..."])
            .current_dir(project_path)
            .output()
            .map_err(|e| AgentError::ValidationError(format!("编译测试失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AgentError::ValidationError(format!(
                "测试编译失败: {}",
                stderr
            )));
        }

        debug!("测试编译成功");
        Ok(())
    }

    /// 运行测试并生成覆盖率报告
    async fn run_tests_with_coverage(
        &self,
        project_path: &PathBuf,
    ) -> Result<PathBuf, AgentError> {
        debug!("运行测试并生成覆盖率报告...");

        let coverage_file = project_path.join("coverage.out");
        let coverage_mode = match self.coverage_mode {
            CoverageMode::Line => "set",
            CoverageMode::Block => "count",
            CoverageMode::Function => "atomic",
        };

        let output = Command::new(&self.go_command)
            .args(&[
                "test",
                "./...",
                "-v",
                &format!("-coverprofile={}", coverage_file.display()),
                &format!("-covermode={}", coverage_mode),
            ])
            .current_dir(project_path)
            .output()
            .map_err(|e| AgentError::ValidationError(format!("运行测试失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("测试运行有问题: {}", stderr);
        }

        // 即使测试失败，覆盖率文件可能还是生成了
        if !coverage_file.exists() {
            return Err(AgentError::ValidationError(
                "覆盖率报告文件未生成".to_string(),
            ));
        }

        debug!("覆盖率报告生成完成: {:?}", coverage_file);
        Ok(coverage_file)
    }

    /// 解析覆盖率报告
    async fn parse_coverage_report(
        &self,
        project_path: &PathBuf,
        coverage_file: &PathBuf,
    ) -> Result<CoverageResult, AgentError> {
        debug!("解析覆盖率报告: {:?}", coverage_file);

        let coverage_content = std::fs::read_to_string(coverage_file)
            .map_err(|e| AgentError::IoError(e))?;

        let mut total_lines = 0;
        let mut covered_lines = 0;

        // 解析覆盖率文件格式：
        // filename:start.column,end.column num_stmts count
        for line in coverage_content.lines() {
            if line.starts_with("mode:") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(num_stmts), Ok(count)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                    total_lines += num_stmts;
                    if count > 0 {
                        covered_lines += num_stmts;
                    }
                }
            }
        }

        let coverage_percentage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        Ok(CoverageResult {
            coverage_percentage,
            covered_lines: covered_lines as usize,
            total_lines: total_lines as usize,
            uncovered_functions: Vec::new(), // 将在后续步骤中填充
            report_file: Some(coverage_file.clone()),
        })
    }

    /// 分析未覆盖的代码
    async fn analyze_uncovered_code(
        &self,
        project_path: &PathBuf,
        mut coverage_result: CoverageResult,
    ) -> Result<CoverageResult, AgentError> {
        debug!("分析未覆盖的代码");

        if let Some(coverage_file) = &coverage_result.report_file {
            // 使用go tool cover分析详细覆盖率
            let output = Command::new(&self.go_command)
                .args(&["tool", "cover", "-func", &coverage_file.display().to_string()])
                .current_dir(project_path)
                .output()
                .map_err(|e| AgentError::ValidationError(format!("分析覆盖率失败: {}", e)))?;

            if output.status.success() {
                let func_coverage = String::from_utf8_lossy(&output.stdout);
                coverage_result.uncovered_functions = self.extract_uncovered_functions(&func_coverage);
            }
        }

        Ok(coverage_result)
    }

    /// 从函数覆盖率报告中提取未覆盖的函数
    fn extract_uncovered_functions(&self, func_coverage: &str) -> Vec<String> {
        let mut uncovered = Vec::new();

        for line in func_coverage.lines() {
            if line.contains("0.0%") {
                // 解析格式：filename:lineno:funcname  0.0%
                if let Some(parts) = line.split_whitespace().next() {
                    if let Some(func_name) = parts.split(':').last() {
                        uncovered.push(func_name.to_string());
                    }
                }
            }
        }

        uncovered
    }

    /// 运行Go测试语法检查
    async fn run_go_test_syntax_check(
        &self,
        project_path: &PathBuf,
        test_file: &PathBuf,
    ) -> Result<bool, AgentError> {
        let output = Command::new(&self.go_command)
            .args(&["test", "-c", test_file.to_str().unwrap()])
            .current_dir(project_path)
            .output()
            .map_err(|e| AgentError::ValidationError(format!("语法检查失败: {}", e)))?;

        Ok(output.status.success())
    }

    /// 统计测试函数数量
    fn count_test_functions(&self, test_code: &str) -> usize {
        let re = Regex::new(r"func\s+(Test\w*|Benchmark\w*)\s*\(").unwrap();
        re.captures_iter(test_code).count()
    }

    /// 统计断言数量
    fn count_assertions(&self, test_code: &str) -> usize {
        let patterns = [
            r"assert\.",
            r"require\.",
            r"\.Error\(",
            r"\.NoError\(",
            r"\.Equal\(",
            r"\.NotEqual\(",
            r"if.*!=.*t\.Error",
            r"if.*==.*t\.Error",
        ];

        let mut count = 0;
        for pattern in &patterns {
            let re = Regex::new(pattern).unwrap();
            count += re.captures_iter(test_code).count();
        }
        count
    }

    /// 检查是否有表驱动测试
    fn has_table_driven_tests(&self, test_code: &str) -> bool {
        let patterns = [
            r"tests\s*:=\s*\[\]",
            r"testCases\s*:=\s*\[\]",
            r"for.*range.*tests",
            r"for.*range.*testCases",
        ];

        patterns.iter().any(|pattern| {
            let re = Regex::new(pattern).unwrap();
            re.is_match(test_code)
        })
    }

    /// 检查是否有错误处理测试
    fn has_error_handling_tests(&self, test_code: &str) -> bool {
        let patterns = [
            r"TestError",
            r"Test.*Error",
            r"\.Error\(",
            r"err\s*!=\s*nil",
            r"wantErr",
        ];

        patterns.iter().any(|pattern| {
            let re = Regex::new(pattern).unwrap();
            re.is_match(test_code)
        })
    }

    /// 检查是否使用了Mock
    fn uses_mocks(&self, test_code: &str) -> bool {
        let patterns = [
            r"mock\.",
            r"Mock",
            r"gomock",
            r"testify/mock",
        ];

        patterns.iter().any(|pattern| {
            let re = Regex::new(pattern).unwrap();
            re.is_match(test_code)
        })
    }

    /// 检查是否有基准测试
    fn has_benchmark_tests(&self, test_code: &str) -> bool {
        let re = Regex::new(r"func\s+Benchmark\w*\s*\(").unwrap();
        re.is_match(test_code)
    }

    /// 计算测试质量评分
    fn calculate_quality_score(&self, report: &TestQualityReport) -> f64 {
        let mut score = 0.0;

        // 基础评分：有测试函数
        if report.test_function_count > 0 {
            score += 20.0;
        }

        // 断言评分：平均每个测试函数有足够的断言
        if report.test_function_count > 0 {
            let assertions_per_test = report.assertion_count as f64 / report.test_function_count as f64;
            score += (assertions_per_test.min(5.0) / 5.0) * 20.0;
        }

        // 表驱动测试
        if report.has_table_driven_tests {
            score += 20.0;
        }

        // 错误处理测试
        if report.has_error_handling_tests {
            score += 20.0;
        }

        // Mock使用
        if report.uses_mocks {
            score += 10.0;
        }

        // 基准测试
        if report.has_benchmark_tests {
            score += 10.0;
        }

        score.min(100.0)
    }
}

/// 测试质量报告
#[derive(Debug, Clone)]
pub struct TestQualityReport {
    /// 测试函数数量
    pub test_function_count: usize,
    /// 断言数量
    pub assertion_count: usize,
    /// 是否有表驱动测试
    pub has_table_driven_tests: bool,
    /// 是否有错误处理测试
    pub has_error_handling_tests: bool,
    /// 是否使用Mock
    pub uses_mocks: bool,
    /// 是否有基准测试
    pub has_benchmark_tests: bool,
    /// 质量评分 (0-100)
    pub quality_score: f64,
}

impl TestQualityReport {
    pub fn new() -> Self {
        Self {
            test_function_count: 0,
            assertion_count: 0,
            has_table_driven_tests: false,
            has_error_handling_tests: false,
            uses_mocks: false,
            has_benchmark_tests: false,
            quality_score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_test_functions() {
        let validator = TestValidator::new();
        let test_code = r#"
            func TestUserHandler(t *testing.T) {}
            func TestGetUser(t *testing.T) {}
            func BenchmarkUserHandler(b *testing.B) {}
            func normalFunction() {}
        "#;

        assert_eq!(validator.count_test_functions(test_code), 3);
    }

    #[test]
    fn test_has_table_driven_tests() {
        let validator = TestValidator::new();
        let test_code = r#"
            func TestExample(t *testing.T) {
                tests := []struct {
                    name string
                    want int
                }{
                    {"test1", 1},
                    {"test2", 2},
                }

                for _, tt := range tests {
                    t.Run(tt.name, func(t *testing.T) {
                        // test logic
                    })
                }
            }
        "#;

        assert!(validator.has_table_driven_tests(test_code));
    }

    #[test]
    fn test_has_error_handling_tests() {
        let validator = TestValidator::new();
        let test_code = r#"
            func TestError(t *testing.T) {
                err := someFunction()
                if err != nil {
                    t.Error("Expected error")
                }
            }
        "#;

        assert!(validator.has_error_handling_tests(test_code));
    }
}