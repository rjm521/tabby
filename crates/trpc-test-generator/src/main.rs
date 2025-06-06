use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn, error};
use tracing_subscriber;

mod agent;
mod generator;
mod validator;
mod types;

use agent::{CoverAgent, AgentCompletion};
use types::{TrpcSchema, GeneratedTests};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "trpc-test-gen")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 分析tRPC项目并生成测试用例
    Generate {
        /// tRPC项目根目录路径
        #[arg(short, long)]
        project_path: PathBuf,

        /// 输出测试文件的目录
        #[arg(short, long, default_value = "./generated_tests")]
        output_dir: PathBuf,

        /// 使用的LLM模型名称
        #[arg(short, long, default_value = "gpt-3.5-turbo")]
        model: String,

        /// LLM API密钥
        #[arg(short, long)]
        api_key: Option<String>,

        /// 是否验证生成的测试覆盖率
        #[arg(long, default_value = "true")]
        validate_coverage: bool,
    },

    /// 分析现有项目的tRPC路由
    Analyze {
        /// tRPC项目根目录路径
        #[arg(short, long)]
        project_path: PathBuf,

        /// 输出分析结果的JSON文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 验证生成的测试覆盖率
    Validate {
        /// 项目路径
        #[arg(short, long)]
        project_path: PathBuf,

        /// 测试文件路径
        #[arg(short, long)]
        test_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("trpc_test_generator=info")
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate {
            project_path,
            output_dir,
            model,
            api_key,
            validate_coverage,
        } => {
            info!("开始为tRPC项目生成测试用例");
            info!("项目路径: {:?}", project_path);
            info!("输出目录: {:?}", output_dir);
            info!("使用模型: {}", model);

            // 创建输出目录
            std::fs::create_dir_all(output_dir)?;

            // 初始化Cover Agent
            let agent = CoverAgent::new(model.clone(), api_key.clone()).await?;

            // 生成测试
            let result = generate_tests_for_project(&agent, project_path, output_dir).await?;

            info!("成功生成 {} 个测试文件", result.test_files.len());

            // 如果需要验证覆盖率
            if *validate_coverage {
                info!("开始验证测试覆盖率...");
                let coverage_result = validate_test_coverage(project_path, &result).await?;
                info!("测试覆盖率: {:.2}%", coverage_result.coverage_percentage);
            }
        }

        Commands::Analyze { project_path, output } => {
            info!("开始分析tRPC项目结构");

            let agent = CoverAgent::new("gpt-3.5-turbo".to_string(), None).await?;
            let schema = agent.analyze_project_structure(project_path).await?;

            if let Some(output_path) = output {
                let json_output = serde_json::to_string_pretty(&schema)?;
                std::fs::write(output_path, json_output)?;
                info!("分析结果已保存到: {:?}", output_path);
            } else {
                println!("{}", serde_json::to_string_pretty(&schema)?);
            }
        }

        Commands::Validate { project_path, test_path } => {
            info!("开始验证测试覆盖率");

            // 这里应该调用实际的覆盖率验证逻辑
            let coverage = run_coverage_analysis(project_path, test_path).await?;
            info!("测试覆盖率: {:.2}%", coverage);
        }
    }

    Ok(())
}

/// 为整个项目生成测试用例
async fn generate_tests_for_project(
    agent: &CoverAgent,
    project_path: &PathBuf,
    output_dir: &PathBuf,
) -> Result<GeneratedTests> {
    // 1. 分析项目结构，识别tRPC路由
    info!("正在分析项目结构...");
    let schema = agent.analyze_project_structure(project_path).await?;

    // 2. 为每个路由生成测试
    let mut all_tests = GeneratedTests {
        test_files: Vec::new(),
        total_routes: schema.routes.len(),
        coverage_info: None,
    };

    for route in &schema.routes {
        info!("为路由 {} 生成测试", route.name);

        // 读取源文件
        let source_content = std::fs::read_to_string(&route.source_file)?;

        // 检查是否已有测试文件
        let existing_tests = find_existing_tests(project_path, &route.name)?;

        // 生成测试
        let generated_test = agent.generate_tests(
            &source_content,
            &existing_tests,
            "",  // 暂时不传入覆盖率报告
            &schema,
        ).await?;

        // 保存生成的测试
        let test_file_path = output_dir.join(format!("{}_test.go", route.name));
        std::fs::write(&test_file_path, &generated_test.test_code)?;

        all_tests.test_files.push(generated_test);
        info!("测试文件已生成: {:?}", test_file_path);
    }

    info!("所有测试生成完成!");
    Ok(all_tests)
}

/// 查找现有的测试文件
fn find_existing_tests(project_path: &PathBuf, route_name: &str) -> Result<String> {
    let test_file_patterns = vec![
        format!("{}_test.go", route_name),
        format!("{}Test.go", route_name),
    ];

    for pattern in test_file_patterns {
        let test_path = project_path.join(pattern);
        if test_path.exists() {
            return Ok(std::fs::read_to_string(test_path)?);
        }
    }

    Ok(String::new())
}

/// 验证测试覆盖率
async fn validate_test_coverage(
    project_path: &PathBuf,
    generated_tests: &GeneratedTests,
) -> Result<types::CoverageResult> {
    use validator::TestValidator;

    let validator = TestValidator::new();
    validator.validate_coverage(project_path, generated_tests).await
        .map_err(|e| anyhow::anyhow!("验证失败: {:?}", e))
}

/// 运行覆盖率分析
async fn run_coverage_analysis(
    project_path: &PathBuf,
    test_path: &PathBuf,
) -> Result<f64> {
    // 执行 go test -cover 命令
    use std::process::Command;

    let output = Command::new("go")
        .args(&["test", "-cover", "-v"])
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        error!("测试执行失败: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("测试执行失败"));
    }

    // 解析覆盖率结果
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_coverage_from_output(&output_str)
}

/// 从go test输出中解析覆盖率
fn parse_coverage_from_output(output: &str) -> Result<f64> {
    use regex::Regex;

    let re = Regex::new(r"coverage:\s*([\d.]+)%")?;

    if let Some(captures) = re.captures(output) {
        let coverage_str = captures.get(1).unwrap().as_str();
        Ok(coverage_str.parse::<f64>()?)
    } else {
        warn!("无法从输出中解析覆盖率信息");
        Ok(0.0)
    }
}