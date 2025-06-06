use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::types::{
    AgentError, TrpcConfig, TrpcField, TrpcRoute, TrpcRouteType, TrpcSchema, TrpcType,
};

/// GoAnalyzer - 专门分析Go代码和tRPC结构
pub struct TrpcAnalyzer {
    /// 正则表达式模式缓存
    patterns: AnalysisPatterns,
}

/// 分析模式集合
struct AnalysisPatterns {
    /// tRPC路由定义模式
    route_pattern: Regex,
    /// 函数定义模式
    func_pattern: Regex,
    /// 结构体定义模式
    struct_pattern: Regex,
    /// 字段定义模式
    field_pattern: Regex,
    /// 包名模式
    package_pattern: Regex,
    /// 导入模式
    import_pattern: Regex,
    /// 注释模式
    comment_pattern: Regex,
}

impl AnalysisPatterns {
    fn new() -> Self {
        Self {
            // 匹配tRPC路由定义，如 r.Query("getUserInfo", getUserInfoHandler)
            route_pattern: Regex::new(r#"r\.(Query|Mutation|Subscription)\s*\(\s*"([^"]+)"\s*,\s*(\w+)\s*\)"#).unwrap(),

            // 匹配函数定义
            func_pattern: Regex::new(r"func\s+(\w+)\s*\(([^)]*)\)\s*(\([^)]*\))?\s*\{").unwrap(),

            // 匹配结构体定义
            struct_pattern: Regex::new(r"type\s+(\w+)\s+struct\s*\{([^}]*)").unwrap(),

            // 匹配结构体字段
            field_pattern: Regex::new(r"(\w+)\s+([^`\n]+)(?:`[^`]*`)?").unwrap(),

            // 匹配包名
            package_pattern: Regex::new(r"package\s+(\w+)").unwrap(),

            // 匹配导入语句
            import_pattern: Regex::new(r#"import\s*(?:\(\s*((?:[^)]*\n)*)\s*\)|"([^"]+)")"#).unwrap(),

            // 匹配注释
            comment_pattern: Regex::new(r"//\s*(.*)").unwrap(),
        }
    }
}

impl TrpcAnalyzer {
    /// 创建新的tRPC分析器
    pub fn new() -> Self {
        Self {
            patterns: AnalysisPatterns::new(),
        }
    }

    /// 分析整个项目
    pub async fn analyze_project(&self, project_path: &PathBuf) -> Result<TrpcSchema, AgentError> {
        info!("开始分析tRPC项目: {:?}", project_path);

        if !project_path.exists() {
            return Err(AgentError::CodeAnalysisError(format!(
                "项目路径不存在: {:?}",
                project_path
            )));
        }

        // 收集所有Go文件
        let go_files = self.collect_go_files(project_path)?;
        info!("找到 {} 个Go文件", go_files.len());

        let mut all_routes = Vec::new();
        let mut all_types = Vec::new();
        let mut project_config = None;

        // 分析每个Go文件
        for file_path in go_files {
            debug!("分析文件: {:?}", file_path);

            let content = fs::read_to_string(&file_path)
                .map_err(|e| AgentError::IoError(e))?;

            // 分析路由
            let mut routes = self.analyze_routes(&content).await?;
            for route in &mut routes {
                route.source_file = file_path.clone();
            }
            all_routes.extend(routes);

            // 分析类型定义
            let types = self.analyze_types(&content)?;
            all_types.extend(types);

            // 分析项目配置（只取第一个找到的）
            if project_config.is_none() {
                if let Ok(config) = self.analyze_config(&content, &file_path) {
                    project_config = Some(config);
                }
            }
        }

        // 构建最终的schema
        let schema = TrpcSchema {
            project_name: self.extract_project_name(project_path),
            routes: all_routes,
            types: all_types,
            config: project_config.unwrap_or_else(|| self.default_config(project_path)),
        };

        info!(
            "项目分析完成: {} 个路由, {} 个类型",
            schema.routes.len(),
            schema.types.len()
        );

        Ok(schema)
    }

    /// 分析源代码中的tRPC路由
    pub async fn analyze_routes(&self, source_code: &str) -> Result<Vec<TrpcRoute>, AgentError> {
        let mut routes = Vec::new();

        // 按行分析，记录行号
        let lines: Vec<&str> = source_code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(captures) = self.patterns.route_pattern.captures(line) {
                let route_type_str = captures.get(1).unwrap().as_str();
                let route_name = captures.get(2).unwrap().as_str();
                let handler_name = captures.get(3).unwrap().as_str();

                let route_type = match route_type_str {
                    "Query" => TrpcRouteType::Query,
                    "Mutation" => TrpcRouteType::Mutation,
                    "Subscription" => TrpcRouteType::Subscription,
                    _ => TrpcRouteType::Query,
                };

                // 尝试推断输入输出类型
                let (input_type, output_type) = self.infer_handler_types(source_code, handler_name)?;

                // 查找路由的文档注释
                let documentation = self.extract_route_documentation(&lines, line_num);

                let route = TrpcRoute {
                    name: route_name.to_string(),
                    route_type,
                    input_type,
                    output_type,
                    handler_name: handler_name.to_string(),
                    source_file: PathBuf::new(), // 会在调用方设置
                    line_range: (line_num + 1, line_num + 1),
                    middlewares: Vec::new(), // TODO: 实现中间件分析
                    documentation,
                };

                routes.push(route);
                debug!("发现tRPC路由: {} ({})", route_name, route_type_str);
            }
        }

        Ok(routes)
    }

    /// 分析Go类型定义
    fn analyze_types(&self, source_code: &str) -> Result<Vec<TrpcType>, AgentError> {
        let mut types = Vec::new();

        for captures in self.patterns.struct_pattern.captures_iter(source_code) {
            let type_name = captures.get(1).unwrap().as_str();
            let struct_body = captures.get(2).unwrap().as_str();

            let fields = self.parse_struct_fields(struct_body)?;

            let trpc_type = TrpcType {
                name: type_name.to_string(),
                go_type: format!("struct {}", type_name),
                fields,
                is_input: self.is_input_type(type_name),
                is_output: self.is_output_type(type_name),
            };

            types.push(trpc_type);
            debug!("发现类型定义: {}", type_name);
        }

        Ok(types)
    }

    /// 解析结构体字段
    fn parse_struct_fields(&self, struct_body: &str) -> Result<Vec<TrpcField>, AgentError> {
        let mut fields = Vec::new();

        for line in struct_body.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if let Some(captures) = self.patterns.field_pattern.captures(line) {
                let field_name = captures.get(1).unwrap().as_str();
                let field_type = captures.get(2).unwrap().as_str().trim();

                let field = TrpcField {
                    name: field_name.to_string(),
                    field_type: field_type.to_string(),
                    required: !field_type.starts_with('*'), // 指针类型通常表示可选
                    validation: None, // TODO: 从标签中提取验证规则
                    description: None, // TODO: 从注释中提取描述
                };

                fields.push(field);
            }
        }

        Ok(fields)
    }

    /// 分析项目配置
    fn analyze_config(&self, source_code: &str, file_path: &PathBuf) -> Result<TrpcConfig, AgentError> {
        let package_name = self.extract_package_name(source_code)
            .unwrap_or_else(|| "main".to_string());

        let import_paths = self.extract_import_paths(source_code);

        Ok(TrpcConfig {
            package_name,
            import_paths,
            global_middlewares: Vec::new(), // TODO: 实现中间件分析
        })
    }

    /// 收集项目中的所有Go文件
    fn collect_go_files(&self, project_path: &PathBuf) -> Result<Vec<PathBuf>, AgentError> {
        let mut go_files = Vec::new();

        for entry in WalkDir::new(project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "go") {
                // 跳过测试文件和vendor目录
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with("_test.go") {
                        continue;
                    }
                }

                if path.components().any(|c| c.as_os_str() == "vendor") {
                    continue;
                }

                go_files.push(path.to_path_buf());
            }
        }

        Ok(go_files)
    }

    /// 从处理函数推断输入输出类型
    fn infer_handler_types(&self, source_code: &str, handler_name: &str) -> Result<(Option<String>, Option<String>), AgentError> {
        // 查找函数定义
        let func_pattern = format!(r"func\s+{}\s*\(([^)]*)\)\s*(\([^)]*\))?", regex::escape(handler_name));
        let re = Regex::new(&func_pattern).unwrap();

        if let Some(captures) = re.captures(source_code) {
            let params = captures.get(1).unwrap().as_str();
            let returns = captures.get(2).map(|m| m.as_str());

            let input_type = self.extract_input_type_from_params(params);
            let output_type = self.extract_output_type_from_returns(returns);

            Ok((input_type, output_type))
        } else {
            warn!("未找到处理函数定义: {}", handler_name);
            Ok((None, None))
        }
    }

    /// 从函数参数中提取输入类型
    fn extract_input_type_from_params(&self, params: &str) -> Option<String> {
        // 简单的启发式规则：查找非ctx参数
        for param in params.split(',') {
            let param = param.trim();
            if !param.contains("context.Context") && !param.contains("ctx") {
                // 提取类型名称
                if let Some(type_name) = param.split_whitespace().last() {
                    return Some(type_name.to_string());
                }
            }
        }
        None
    }

    /// 从函数返回值中提取输出类型
    fn extract_output_type_from_returns(&self, returns: Option<&str>) -> Option<String> {
        if let Some(returns) = returns {
            let returns = returns.trim_matches(|c| c == '(' || c == ')');
            // 假设第一个返回值是数据，第二个是error
            if let Some(first_return) = returns.split(',').next() {
                let first_return = first_return.trim();
                if first_return != "error" {
                    return Some(first_return.to_string());
                }
            }
        }
        None
    }

    /// 提取路由的文档注释
    fn extract_route_documentation(&self, lines: &[&str], route_line: usize) -> Option<String> {
        let mut docs = Vec::new();

        // 向上查找注释行
        for i in (0..route_line).rev() {
            let line = lines[i].trim();
            if line.starts_with("//") {
                if let Some(captures) = self.patterns.comment_pattern.captures(line) {
                    docs.insert(0, captures.get(1).unwrap().as_str().trim().to_string());
                }
            } else if !line.is_empty() {
                break; // 遇到非空非注释行就停止
            }
        }

        if docs.is_empty() {
            None
        } else {
            Some(docs.join(" "))
        }
    }

    /// 提取包名
    fn extract_package_name(&self, source_code: &str) -> Option<String> {
        if let Some(captures) = self.patterns.package_pattern.captures(source_code) {
            Some(captures.get(1).unwrap().as_str().to_string())
        } else {
            None
        }
    }

    /// 提取导入路径
    fn extract_import_paths(&self, source_code: &str) -> Vec<String> {
        let mut imports = Vec::new();

        for captures in self.patterns.import_pattern.captures_iter(source_code) {
            if let Some(multi_import) = captures.get(1) {
                // 多行导入
                for line in multi_import.as_str().lines() {
                    let line = line.trim();
                    if let Some(import_path) = self.extract_single_import(line) {
                        imports.push(import_path);
                    }
                }
            } else if let Some(single_import) = captures.get(2) {
                // 单行导入
                imports.push(single_import.as_str().to_string());
            }
        }

        imports
    }

    /// 提取单个导入路径
    fn extract_single_import(&self, line: &str) -> Option<String> {
        let re = Regex::new(r#""([^"]+)""#).unwrap();
        if let Some(captures) = re.captures(line) {
            Some(captures.get(1).unwrap().as_str().to_string())
        } else {
            None
        }
    }

    /// 判断是否为输入类型
    fn is_input_type(&self, type_name: &str) -> bool {
        type_name.ends_with("Request") ||
        type_name.ends_with("Input") ||
        type_name.ends_with("Req")
    }

    /// 判断是否为输出类型
    fn is_output_type(&self, type_name: &str) -> bool {
        type_name.ends_with("Response") ||
        type_name.ends_with("Output") ||
        type_name.ends_with("Resp") ||
        type_name.ends_with("Result")
    }

    /// 提取项目名称
    fn extract_project_name(&self, project_path: &PathBuf) -> String {
        project_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// 创建默认配置
    fn default_config(&self, project_path: &PathBuf) -> TrpcConfig {
        TrpcConfig {
            package_name: "main".to_string(),
            import_paths: vec![
                "context".to_string(),
                "fmt".to_string(),
                "testing".to_string(),
            ],
            global_middlewares: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_routes() {
        let analyzer = TrpcAnalyzer::new();
        let source_code = r#"
            package main

            func setupRoutes() {
                r.Query("getUserInfo", getUserInfoHandler)
                r.Mutation("updateUser", updateUserHandler)
            }
        "#;

        let routes = analyzer.analyze_routes(source_code).await.unwrap();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].name, "getUserInfo");
        assert_eq!(routes[1].name, "updateUser");
    }

    #[test]
    fn test_analyze_types() {
        let analyzer = TrpcAnalyzer::new();
        let source_code = r#"
            type UserRequest struct {
                ID   int    `json:"id"`
                Name string `json:"name"`
            }

            type UserResponse struct {
                User User `json:"user"`
                Success bool `json:"success"`
            }
        "#;

        let types = analyzer.analyze_types(source_code).unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "UserRequest");
        assert_eq!(types[1].name, "UserResponse");
        assert!(types[0].is_input);
        assert!(types[1].is_output);
    }
}