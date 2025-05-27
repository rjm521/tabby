use axum::{
    extract::{Path, Query},
    response::sse::{Event, Sse},
    Json, Router,
    routing::{get, post, delete},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use futures::stream::{self, Stream};
use futures::StreamExt;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tabby_index;
use crate::services::embedding;
use tabby_common::config::CodeRepository;
use std::io::Cursor;
use zip::ZipArchive;
use tantivy::{
    DocAddress, DocSet,
    schema::{Term, Value},
    Index,
    TERMINATED, TantivyDocument, Document,
    query::{QueryParser, FuzzyTermQuery},
    collector::TopDocs,
};
use tabby_common::index::IndexSchema;
use chrono::{Duration, Utc};
use std::process::Command;
use std::path::Path as StdPath;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IndexInfo {
    /// 索引分片数量
    #[schema(example = 3)]
    num_segments: usize,
    /// 索引占用总字节数
    #[schema(example = 1048576)]
    total_bytes: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    /// 语料库名称
    #[schema(example = "repo1")]
    corpus: String,
    /// 文档内容
    #[schema(example = "{\"file\":\"src/main.rs\",\"content\":\"fn main() {}\"}")]
    content: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexResponse {
    /// 操作是否成功
    #[schema(example = true)]
    success: bool,
    /// 索引任务ID
    #[schema(example = "idx_abc123def456")]
    index_id: String,
    /// 可选的消息
    #[schema(example = "Index creation started successfully")]
    message: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    /// 搜索查询
    #[schema(example = "function main")]
    query: String,
    /// 编程语言过滤
    #[schema(example = "rust")]
    language: Option<String>,
    /// 返回结果数量限制
    #[schema(example = 10)]
    limit: Option<usize>,
    /// 偏移量
    #[schema(example = 0)]
    offset: Option<usize>,
    /// 文件路径过滤
    #[schema(example = "src/")]
    file_path: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// 文件路径
    #[schema(example = "src/main.rs")]
    file_path: String,
    /// 匹配的代码片段
    #[schema(example = "fn main() { println!(\"Hello, world!\"); }")]
    content: String,
    /// 开始行号
    #[schema(example = 1)]
    start_line: Option<usize>,
    /// 结束行号
    #[schema(example = 3)]
    end_line: Option<usize>,
    /// 匹配分数
    #[schema(example = 0.95)]
    score: f32,
    /// 编程语言
    #[schema(example = "rust")]
    language: String,
    /// Git URL
    #[schema(example = "https://github.com/user/repo.git")]
    git_url: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// 搜索结果列表
    results: Vec<SearchResult>,
    /// 总结果数量
    total: usize,
    /// 查询时间（毫秒）
    query_time_ms: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchQuery {
    /// 文件名查询
    #[schema(example = "main.rs")]
    q: String,
    /// 返回结果数量限制
    #[schema(example = 10)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    /// 索引ID
    #[schema(example = "idx_abc123def456")]
    index_id: String,
    /// 索引状态
    #[schema(example = "ready")]
    status: String,
    /// 文档数量
    #[schema(example = 1500)]
    document_count: usize,
    /// 索引大小（字节）
    #[schema(example = 1048576)]
    size_bytes: u64,
    /// 最后更新时间
    #[schema(example = "2024-03-21T10:30:00Z")]
    last_updated: String,
    /// 索引版本
    #[schema(example = "1.0")]
    version: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IndexConfig {
    /// 最大文件大小（KB）
    #[schema(example = 1024)]
    max_file_size: usize,
    /// 包含文件模式
    #[schema(example = "[\"**/*.rs\", \"**/*.py\"]")]
    include_patterns: Vec<String>,
    /// 排除文件模式
    #[schema(example = "[\"target/**\", \"**/*.pyc\"]")]
    exclude_patterns: Vec<String>,
    /// 支持的编程语言
    #[schema(example = "[\"rust\", \"python\", \"javascript\"]")]
    languages: Vec<String>,
    /// 是否启用语义搜索
    #[schema(example = true)]
    enable_semantic_search: bool,
    /// 索引更新间隔（秒）
    #[schema(example = 3600)]
    update_interval_seconds: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigValidationResponse {
    /// 配置是否有效
    #[schema(example = true)]
    valid: bool,
    /// 验证错误信息
    #[schema(example = "[]")]
    errors: Vec<String>,
    /// 验证警告信息
    #[schema(example = "[]")]
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRequest {
    /// 代码内容
    #[schema(example = "fn main() { println!(\"Hello, world!\"); }")]
    content: String,
    /// 文件路径
    #[schema(example = "src/main.rs")]
    filepath: String,
    /// 编程语言
    #[schema(example = "rust")]
    language: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResponse {
    /// 代码复杂度
    #[schema(example = 2)]
    complexity: usize,
    /// 函数数量
    #[schema(example = 1)]
    function_count: usize,
    /// 类数量
    #[schema(example = 0)]
    class_count: usize,
    /// 代码行数
    #[schema(example = 3)]
    lines_of_code: usize,
    /// 建议的索引标签
    #[schema(example = "[\"main\", \"function\", \"rust\"]")]
    suggested_tags: Vec<String>,
    /// 代码质量评分（0-100）
    #[schema(example = 85)]
    quality_score: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IndexSuggestion {
    /// 建议类型
    #[schema(example = "optimization")]
    suggestion_type: String,
    /// 建议描述
    #[schema(example = "Consider adding more file types to indexing")]
    description: String,
    /// 优先级
    #[schema(example = "medium")]
    priority: String,
    /// 相关索引
    #[schema(example = "idx_abc123def456")]
    related_index: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateRequest {
    /// 批量创建的源列表
    sources: Vec<CreateIndexRequest>,
    /// 并发数量
    #[schema(example = 2)]
    concurrency: Option<usize>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchStatus {
    /// 批次ID
    #[schema(example = "batch-123")]
    batch_id: String,
    /// 总任务数
    #[schema(example = 5)]
    total_tasks: usize,
    /// 已完成任务数
    #[schema(example = 3)]
    completed_tasks: usize,
    /// 失败任务数
    #[schema(example = 1)]
    failed_tasks: usize,
    /// 批次状态
    #[schema(example = "running")]
    status: String,
    /// 开始时间
    #[schema(example = "2024-03-21T10:30:00Z")]
    start_time: String,
    /// 预估完成时间
    #[schema(example = "2024-03-21T11:30:00Z")]
    estimated_completion: Option<String>,
}

#[derive(Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    /// 扫描的目录数量
    directories_scanned: usize,
    /// 跳过的文件数量
    files_skipped: usize,
    /// 索引的代码行数
    lines_indexed: usize,
    /// 生成的代码块数量
    chunks_generated: usize,
    /// 平均文件大小 (KB)
    avg_file_size_kb: f32,
}

#[derive(Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexingProgress {
    /// 索引任务ID
    index_id: String,
    /// 总文件数
    total_files: usize,
    /// 已处理文件数
    processed_files: usize,
    /// 已更新chunks数
    updated_chunks: usize,
    /// 进度百分比
    progress_percentage: f32,
    /// 处理状态 (initializing, downloading, extracting, cloning, indexing, completed, failed)
    status: String,
    /// 具体的状态消息
    status_msg: String,
    /// 当前正在处理的文件路径
    current_file: Option<String>,
    /// 索引开始时间
    start_time: Option<String>,
    /// 预估完成时间
    estimated_completion: Option<String>,
    /// 处理速度 (文件/秒)
    processing_rate: Option<f32>,
    /// 当前阶段
    current_phase: String,
    /// 索引统计信息
    index_stats: Option<IndexStats>,
}

// 用于保持临时目录生命周期的结构
struct TempDirKeeper {
    _temp_dir: tempfile::TempDir,
    path: String,
}

pub fn index_router() -> Router {
    Router::new()
        .route("/v1/index/info", get(get_index_info))
        .route("/v1/index/documents/{corpus}", get(get_documents))
        .route("/v1/index/create", post(create_index))
        // 搜索功能
        .route("/v1/index/search", post(search_code))
        .route("/v1/index/search/files", get(search_files))
        .route("/v1/index/search/semantic", post(semantic_search))
        // 索引管理
        .route("/v1/index/{indexId}/status", get(get_index_status))
        .route("/v1/index/{indexId}", delete(delete_index))
        .route("/v1/index/{indexId}/rebuild", post(rebuild_index))
        // 配置管理
        .route("/v1/index/config", get(get_index_config))
        .route("/v1/index/config/validate", post(validate_config))
        // 智能分析
        .route("/v1/index/analyze", post(analyze_code))
        .route("/v1/index/suggestions", get(get_suggestions))
        // 批量操作
        .route("/v1/index/batch/create", post(batch_create_index))
        .route("/v1/index/batch/{batch_id}/status", get(get_batch_status))
}

#[utoipa::path(
    get,
    path = "/v1/index/info",
    tag = "index",
    operation_id = "get_index_info",
    responses(
        (status = 200, description = "索引信息", body = IndexInfo, example = json!({"num_segments": 3, "total_bytes": 1048576})),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_index_info() -> Json<IndexInfo> {
    let index = Index::open_in_dir(tabby_common::path::index_dir()).unwrap();
    let searcher = index.reader().unwrap().searcher();
    let segments = searcher.segment_readers();
    let space_usage = searcher.space_usage().unwrap();

    let total_bytes = space_usage.total().to_string().parse::<u64>().unwrap_or(0);

    Json(IndexInfo {
        num_segments: segments.len(),
        total_bytes,
    })
}

#[utoipa::path(
    get,
    path = "/v1/index/documents/{corpus}",
    tag = "index",
    operation_id = "get_documents",
    responses(
        (status = 200, description = "文档列表", body = Vec<DocumentInfo>, example = json!([
            {"corpus": "repo1", "content": "{\"file\":\"src/main.rs\",\"content\":\"fn main() {}\"}"}
        ])),
    ),
    params(
        ("corpus" = String, Path, description = "语料库名称"),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_documents(Path(corpus): Path<String>) -> Json<Vec<DocumentInfo>> {
    let index = Index::open_in_dir(tabby_common::path::index_dir()).unwrap();
    let searcher = index.reader().unwrap().searcher();
    let schema = IndexSchema::instance();

    let mut documents = Vec::new();
    let mut count = 0;
    'outer: for (segment_ordinal, segment_reader) in searcher.segment_readers().iter().enumerate() {
        let Ok(inverted_index) = segment_reader.inverted_index(schema.field_corpus) else {
            continue;
        };

        let term_corpus = Term::from_field_text(schema.field_corpus, &corpus);
        let Ok(Some(mut postings)) =
            inverted_index.read_postings(&term_corpus, tantivy::schema::IndexRecordOption::Basic)
        else {
            continue;
        };

        let mut doc_id = postings.doc();
        while doc_id != TERMINATED {
            if !segment_reader.is_deleted(doc_id) {
                let doc_address = DocAddress::new(segment_ordinal as u32, doc_id);
                let doc: TantivyDocument = searcher.doc(doc_address).unwrap();

                let json_value = to_json_value(doc, &schema.schema);
                documents.push(DocumentInfo {
                    corpus: corpus.clone(),
                    content: json_value.to_string(),
                });

                count += 1;
                if count >= 10 {
                    break 'outer;
                }
            }
            doc_id = postings.advance();
        }
    }

    Json(documents)
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexRequest {
    /// 仓库地址或远程zip文件URL
    #[schema(example = "https://github.com/TabbyML/tabby.git")]
    source: String,
    /// 是否为远程zip文件
    #[schema(example = false)]
    is_remote_zip: Option<bool>,
    /// 索引名称
    #[schema(example = "tabby-index")]
    name: Option<String>,
    /// 语言
    #[schema(example = "rust")]
    language: Option<String>,
    /// 最大文件大小
    #[schema(example = 1024)]
    max_file_size: Option<usize>,
    /// 排除文件
    #[schema(example = "[\"*.md\"]")]
    exclude: Vec<String>,
    /// 包含文件
    #[schema(example = "[\"src/**\"]")]
    include: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/index/create",
    tag = "index",
    operation_id = "create_index",
    request_body = CreateIndexRequest,
    responses(
        (status = 200, description = "创建索引进度", body = String, content_type = "text/event-stream"),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn create_index(Json(request): Json<CreateIndexRequest>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel(100);

    // 生成唯一的索引ID
    let index_id = format!("idx_{}", Uuid::new_v4().simple().to_string()[..12].to_lowercase());

    // 立即发送初始响应
    let initial_progress = IndexingProgress {
        index_id: index_id.clone(),
        total_files: 0,
        processed_files: 0,
        updated_chunks: 0,
        progress_percentage: 0.0,
        status: "initializing".to_string(),
        status_msg: "任务已接收，正在初始化...".to_string(),
        current_file: None,
        start_time: Some(chrono::Utc::now().to_rfc3339()),
        estimated_completion: None,
        processing_rate: None,
        current_phase: "initializing".to_string(),
        index_stats: None,
    };

    let _ = tx.try_send(initial_progress);

    tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        let start_time_str = chrono::Utc::now().to_rfc3339();

        let mut progress = IndexingProgress {
            index_id: index_id.clone(),
            total_files: 0,
            processed_files: 0,
            updated_chunks: 0,
            progress_percentage: 0.0,
            status: "initializing".to_string(),
            status_msg: "准备中...".to_string(),
            current_file: None,
            start_time: Some(start_time_str.clone()),
            estimated_completion: None,
            processing_rate: None,
            current_phase: "initializing".to_string(),
            index_stats: None,
        };

        // 用于保持临时目录生命周期
        let _temp_keeper: Option<TempDirKeeper>;

        let repository = if request.is_remote_zip.unwrap_or(false) {
            // 优化的远程文件下载，带进度反馈
            progress.status = "downloading".to_string();
            progress.status_msg = "正在连接到远程服务器...".to_string();
            progress.current_phase = "downloading".to_string();
            progress.current_file = Some(request.source.clone());
            let _ = tx.send(progress.clone()).await;

            let client = Client::new();
            match client.head(&request.source).send().await {
                Ok(response) => {
                    let content_length = response.headers()
                        .get("content-length")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);

                    progress.status = "downloading".to_string();
                    progress.status_msg = format!("开始下载文件 ({:.2} MB)...", content_length as f64 / 1024.0 / 1024.0);
                    progress.progress_percentage = 5.0;
                    let _ = tx.send(progress.clone()).await;

                    match client.get(&request.source).send().await {
                        Ok(response) => {
                            // 直接获取所有字节，然后显示进度
                            progress.status = "downloading".to_string();
                            progress.status_msg = "正在下载文件内容...".to_string();
                            progress.progress_percentage = 10.0;
                            let _ = tx.send(progress.clone()).await;

                            match response.bytes().await {
                                Ok(bytes) => {
                                    progress.status = "extracting".to_string();
                                    progress.status_msg = "下载完成，正在解压文件...".to_string();
                                    progress.current_phase = "extracting".to_string();
                                    progress.progress_percentage = 20.0;
                                    let _ = tx.send(progress.clone()).await;

                                    // 使用blocking task处理zip文件，避免Send问题
                                    let tx_clone = tx.clone();
                                    let bytes_clone = bytes.clone();
                                    let name_clone = request.name.clone();

                                    match tokio::task::spawn_blocking(move || {
                                        let cursor = Cursor::new(bytes_clone);
                                        let mut archive = ZipArchive::new(cursor)?;
                                        let temp_dir = tempfile::tempdir()?;
                                        let total_files = archive.len();

                                        for i in 0..total_files {
                                            let mut file = archive.by_index(i)?;

                                            if file.name().ends_with('/') {
                                                std::fs::create_dir_all(temp_dir.path().join(file.name()))?;
                                            } else {
                                                if let Some(parent) = StdPath::new(file.name()).parent() {
                                                    std::fs::create_dir_all(temp_dir.path().join(parent))?;
                                                }
                                                let mut outfile = std::fs::File::create(temp_dir.path().join(file.name()))?;
                                                std::io::copy(&mut file, &mut outfile)?;
                                            }
                                        }

                                        Ok::<(tempfile::TempDir, usize), Box<dyn std::error::Error + Send + Sync>>((temp_dir, total_files))
                                    }).await {
                                        Ok(Ok((temp_dir, total_files))) => {
                                            progress.status = "extracting".to_string();
                                            progress.status_msg = format!("解压完成，共提取 {} 个文件", total_files);
                                            progress.progress_percentage = 30.0;
                                            progress.current_file = None;
                                            let _ = tx.send(progress.clone()).await;

                                            // 直接使用本地路径，不添加file://前缀
                                            let local_path = temp_dir.path().to_string_lossy().to_string();
                                            _temp_keeper = Some(TempDirKeeper {
                                                _temp_dir: temp_dir,
                                                path: local_path.clone(),
                                            });

                                            CodeRepository {
                                                git_url: local_path,
                                                source_id: name_clone.unwrap_or_else(|| index_id.clone()),
                                            }
                                        }
                                        Ok(Err(e)) => {
                                            progress.status = "failed".to_string();
                                            progress.status_msg = format!("解压失败: {}", e);
                                            let _ = tx.send(progress).await;
                                            return;
                                        }
                                        Err(e) => {
                                            progress.status = "failed".to_string();
                                            progress.status_msg = format!("解压任务失败: {}", e);
                                            let _ = tx.send(progress).await;
                                            return;
                                        }
                                    }
                                }
                                Err(e) => {
                                    progress.status = "failed".to_string();
                                    progress.status_msg = format!("下载失败: {}", e);
                                    let _ = tx.send(progress).await;
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            progress.status = "failed".to_string();
                            progress.status_msg = format!("连接服务器失败: {}", e);
                            let _ = tx.send(progress).await;
                            return;
                        }
                    }
                }
                Err(e) => {
                    progress.status = "failed".to_string();
                    progress.status_msg = format!("连接服务器失败: {}", e);
                    let _ = tx.send(progress).await;
                    return;
                }
            }
        } else {
            // 优化的Git clone，带进度反馈
            if request.source.starts_with("http") || request.source.starts_with("git") {
                progress.status = "cloning".to_string();
                progress.status_msg = "正在验证Git仓库...".to_string();
                progress.current_phase = "cloning".to_string();
                progress.current_file = Some(request.source.clone());
                let _ = tx.send(progress.clone()).await;

                let temp_dir = tempfile::tempdir().unwrap();
                let clone_path = temp_dir.path().join("repo");

                progress.status = "cloning".to_string();
                progress.status_msg = "开始克隆Git仓库...".to_string();
                progress.progress_percentage = 5.0;
                let _ = tx.send(progress.clone()).await;

                // 使用git命令进行克隆，并捕获进度
                let mut cmd = Command::new("git");
                cmd.args(&["clone", "--progress", &request.source, clone_path.to_str().unwrap()]);

                match cmd.output() {
                    Ok(output) => {
                        if output.status.success() {
                            progress.status = "cloning".to_string();
                            progress.status_msg = "Git仓库克隆完成".to_string();
                            progress.progress_percentage = 30.0;
                            progress.current_file = None;
                            let _ = tx.send(progress.clone()).await;

                            // 直接使用本地路径，不添加file://前缀
                            let local_path = clone_path.to_string_lossy().to_string();
                            _temp_keeper = Some(TempDirKeeper {
                                _temp_dir: temp_dir,
                                path: local_path.clone(),
                            });

                            CodeRepository {
                                git_url: local_path,
                                source_id: request.name.unwrap_or_else(|| index_id.clone()),
                            }
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            progress.status = "failed".to_string();
                            progress.status_msg = format!("Git克隆失败: {}", error_msg);
                            let _ = tx.send(progress).await;
                            return;
                        }
                    }
                    Err(e) => {
                        progress.status = "failed".to_string();
                        progress.status_msg = format!("执行Git命令失败: {}", e);
                        let _ = tx.send(progress).await;
                        return;
                    }
                }
            } else {
                // 本地路径处理
                progress.status = "initializing".to_string();
                progress.status_msg = "验证本地路径...".to_string();
                progress.progress_percentage = 30.0;
                let _ = tx.send(progress.clone()).await;

                // 对于用户提供的本地路径，确保使用file://前缀
                let git_url = if request.source.starts_with("file://") {
                    request.source.clone()
                } else if StdPath::new(&request.source).is_absolute() {
                    format!("file://{}", request.source)
                } else {
                    // 相对路径转换为绝对路径
                    match std::env::current_dir() {
                        Ok(current_dir) => {
                            let absolute_path = current_dir.join(&request.source);
                            format!("file://{}", absolute_path.to_string_lossy())
                        }
                        Err(_) => {
                            progress.status = "failed".to_string();
                            progress.status_msg = "无法获取当前工作目录".to_string();
                            let _ = tx.send(progress).await;
                            return;
                        }
                    }
                };

                _temp_keeper = None;

                CodeRepository {
                    git_url,
                    source_id: request.name.unwrap_or_else(|| index_id.clone()),
                }
            }
        };

        // 获取 embedding 配置
        progress.status = "initializing".to_string();
        progress.status_msg = "初始化embedding模型...".to_string();
        progress.current_phase = "initializing".to_string();
        progress.current_file = None;
        progress.progress_percentage = 35.0;
        let _ = tx.send(progress.clone()).await;

        let config = match tabby_common::config::Config::load() {
            Ok(config) => config.model.embedding,
            Err(e) => {
                progress.status = "failed".to_string();
                progress.status_msg = format!("配置加载失败: {}", e);
                let _ = tx.send(progress).await;
                return;
            }
        };

        let embedding = embedding::create(&config).await;
        let mut indexer = tabby_index::public::CodeIndexer::default();

        progress.status = "initializing".to_string();
        progress.status_msg = "Embedding模型初始化完成".to_string();
        progress.current_phase = "indexing".to_string();
        progress.progress_percentage = 40.0;
        let _ = tx.send(progress.clone()).await;

        // 设置进度回调
        let tx_clone = tx.clone();
        let start_time_clone = start_time;
        let start_time_str_clone = start_time_str.clone();
        let index_id_clone = index_id.clone();
        indexer.set_progress_callback(Box::new(move |total_files, processed_files, updated_chunks| {
            let elapsed = start_time_clone.elapsed().as_secs_f32();
            // 索引进度占总进度的60%（40%-100%）
            let index_progress = if total_files > 0 {
                (processed_files as f32 / total_files as f32) * 60.0
            } else {
                0.0
            };
            let total_progress = 40.0 + index_progress;

            let processing_rate = if elapsed > 0.0 {
                processed_files as f32 / elapsed
            } else {
                0.0
            };

            let estimated_completion = if processing_rate > 0.0 && processed_files < total_files {
                let remaining_files = total_files - processed_files;
                let estimated_seconds = remaining_files as f32 / processing_rate;
                let completion_time = Utc::now() + Duration::seconds(estimated_seconds as i64);
                Some(completion_time.to_rfc3339())
            } else {
                None
            };

            let (status, status_msg) = if processed_files == 0 {
                ("indexing".to_string(), "扫描文件中...".to_string())
            } else if processed_files < total_files {
                ("indexing".to_string(), format!("正在索引文件 ({}/{}) - 已生成 {} 个代码块", processed_files, total_files, updated_chunks))
            } else {
                ("indexing".to_string(), format!("文件处理完成 - 共处理 {} 个文件，生成 {} 个代码块", total_files, updated_chunks))
            };

            // 计算索引统计信息
            let index_stats = if total_files > 0 {
                Some(IndexStats {
                    directories_scanned: 1, // 简化统计
                    files_skipped: 0,
                    lines_indexed: updated_chunks * 10, // 估算代码行数
                    chunks_generated: updated_chunks,
                    avg_file_size_kb: 5.0, // 估算平均文件大小
                })
            } else {
                None
            };

            let progress = IndexingProgress {
                index_id: index_id_clone.clone(),
                total_files,
                processed_files,
                updated_chunks,
                progress_percentage: total_progress,
                status,
                status_msg,
                current_file: None,
                start_time: Some(start_time_str_clone.clone()),
                estimated_completion,
                processing_rate: Some(processing_rate),
                current_phase: "indexing".to_string(),
                index_stats,
            };

            let _ = tx_clone.try_send(progress);
        }));

        // 开始索引
        progress.status = "indexing".to_string();
        progress.status_msg = "开始建立索引...".to_string();
        progress.progress_percentage = 40.0;
        let _ = tx.send(progress.clone()).await;

        match indexer.refresh(embedding, &repository).await {
            Ok(_) => {
                let elapsed = start_time.elapsed().as_secs_f32();
                progress.status = "completed".to_string();
                progress.status_msg = format!("索引创建完成，耗时 {:.1} 秒", elapsed);
                progress.progress_percentage = 100.0;
                progress.current_file = None;
                progress.estimated_completion = None;
                progress.processing_rate = Some(progress.processed_files as f32 / elapsed.max(1.0));
                let _ = tx.send(progress).await;
            }
            Err(e) => {
                progress.status = "failed".to_string();
                progress.status_msg = format!("索引创建失败: {}", e);
                progress.current_file = None;
                progress.estimated_completion = None;
                let _ = tx.send(progress).await;
            }
        }

        // 保持临时目录的生命周期直到索引完成
        drop(_temp_keeper);
    });

    let stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Some(progress) => {
                // 只发送实际的进度数据，不发送keep-alive
                let event = Event::default()
                    .json_data(progress)
                    .unwrap();
                Some((Ok(event), rx))
            }
            None => None,
        }
    });

    Sse::new(stream)
}

#[utoipa::path(
    post,
    path = "/v1/index/search",
    tag = "index",
    operation_id = "search_code",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "搜索结果", body = SearchResponse),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn search_code(Json(request): Json<SearchRequest>) -> Json<SearchResponse> {
    let start_time = std::time::Instant::now();

    let index = Index::open_in_dir(tabby_common::path::index_dir()).unwrap();
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let schema = IndexSchema::instance();

    // 构建查询
    let query_parser = QueryParser::for_index(&index, vec![schema.field_chunk_tokens]);
    let query = query_parser.parse_query(&request.query).unwrap();

    let limit = request.limit.unwrap_or(10);
    let offset = request.offset.unwrap_or(0);

    let top_docs = searcher.search(&query, &TopDocs::with_limit(limit + offset)).unwrap();

    let mut results = Vec::new();
    for (score, doc_address) in top_docs.iter().skip(offset) {
        let doc = searcher.doc(*doc_address).unwrap();

        // 提取文档字段
        if let Some(result) = extract_search_result(&doc, *score, &schema) {
            // 应用语言过滤
            if let Some(ref lang) = request.language {
                if result.language != *lang {
                    continue;
                }
            }

            // 应用文件路径过滤
            if let Some(ref path_filter) = request.file_path {
                if !result.file_path.contains(path_filter) {
                    continue;
                }
            }

            results.push(result);
        }
    }

    let query_time_ms = start_time.elapsed().as_millis() as u64;

    Json(SearchResponse {
        results,
        total: top_docs.len(),
        query_time_ms,
    })
}

#[utoipa::path(
    get,
    path = "/v1/index/search/files",
    tag = "index",
    operation_id = "search_files",
    params(
        ("q" = String, Query, description = "文件名查询"),
        ("limit" = Option<usize>, Query, description = "返回结果数量限制"),
    ),
    responses(
        (status = 200, description = "文件搜索结果", body = Vec<String>),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn search_files(Query(params): Query<FileSearchQuery>) -> Json<Vec<String>> {
    let index = Index::open_in_dir(tabby_common::path::index_dir()).unwrap();
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let schema = IndexSchema::instance();

    // 使用模糊查询搜索文件路径
    let term = Term::from_field_text(schema.field_chunk_attributes, &params.q);
    let fuzzy_query = FuzzyTermQuery::new(term, 2, true);

    let limit = params.limit.unwrap_or(10);
    let top_docs = searcher.search(&fuzzy_query, &TopDocs::with_limit(limit)).unwrap();

    let mut file_paths = Vec::new();
    for (_score, doc_address) in top_docs {
        let doc = searcher.doc(doc_address).unwrap();
        if let Some(path) = extract_file_path(&doc, &schema) {
            if !file_paths.contains(&path) {
                file_paths.push(path);
            }
        }
    }

    Json(file_paths)
}

#[utoipa::path(
    post,
    path = "/v1/index/search/semantic",
    tag = "index",
    operation_id = "semantic_search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "语义搜索结果", body = SearchResponse),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn semantic_search(Json(request): Json<SearchRequest>) -> Json<SearchResponse> {
    // TODO: 实现基于embedding的语义搜索
    // 这里先返回一个基础的实现
    search_code(Json(request)).await
}

#[utoipa::path(
    get,
    path = "/v1/index/{indexId}/status",
    tag = "index",
    operation_id = "get_index_status",
    params(
        ("indexId" = String, Path, description = "索引ID"),
    ),
    responses(
        (status = 200, description = "索引状态", body = IndexStatus),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_index_status(Path(index_id): Path<String>) -> Json<IndexStatus> {
    let index = Index::open_in_dir(tabby_common::path::index_dir()).unwrap();
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();

    // 计算文档数量
    let mut document_count = 0;
    for segment_reader in searcher.segment_readers() {
        document_count += segment_reader.num_docs() as usize;
    }

    // 计算索引大小
    let space_usage = searcher.space_usage().unwrap();
    let size_bytes = space_usage.total().to_string().parse::<u64>().unwrap_or(0);

    Json(IndexStatus {
        index_id,
        status: "ready".to_string(),
        document_count,
        size_bytes,
        last_updated: Utc::now().to_rfc3339(),
        version: "1.0".to_string(),
    })
}

#[utoipa::path(
    delete,
    path = "/v1/index/{indexId}",
    tag = "index",
    operation_id = "delete_index",
    params(
        ("indexId" = String, Path, description = "索引ID"),
    ),
    responses(
        (status = 200, description = "删除成功", body = CreateIndexResponse),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn delete_index(Path(index_id): Path<String>) -> Json<CreateIndexResponse> {
    // TODO: 实现索引删除逻辑
    Json(CreateIndexResponse {
        success: true,
        index_id: format!("del_{}", Uuid::new_v4().simple().to_string()[..12].to_lowercase()),
        message: Some(format!("Index '{}' deleted successfully", index_id)),
    })
}

#[utoipa::path(
    post,
    path = "/v1/index/{indexId}/rebuild",
    tag = "index",
    operation_id = "rebuild_index",
    params(
        ("indexId" = String, Path, description = "索引ID"),
    ),
    responses(
        (status = 200, description = "重建索引进度", body = String, content_type = "text/event-stream"),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn rebuild_index(Path(index_id): Path<String>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // TODO: 实现索引重建逻辑，类似create_index但针对现有索引
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let rebuild_id = format!("rebuild_{}", Uuid::new_v4().simple().to_string()[..12].to_lowercase());
        let progress = IndexingProgress {
            index_id: rebuild_id,
            total_files: 0,
            processed_files: 0,
            updated_chunks: 0,
            progress_percentage: 0.0,
            status: format!("开始重建索引: {}", index_id),
            status_msg: "".to_string(),
            current_file: None,
            start_time: Some(Utc::now().to_rfc3339()),
            estimated_completion: None,
            processing_rate: None,
            current_phase: "initializing".to_string(),
            index_stats: None,
        };
        let _ = tx.send(progress).await;
    });

    let stream = ReceiverStream::new(rx);
    Sse::new(stream.map(|progress| {
        Ok(Event::default().data(serde_json::to_string(&progress).unwrap()))
    }))
}

#[utoipa::path(
    get,
    path = "/v1/index/config",
    tag = "index",
    operation_id = "get_index_config",
    responses(
        (status = 200, description = "索引配置", body = IndexConfig),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_index_config() -> Json<IndexConfig> {
    Json(IndexConfig {
        max_file_size: 1024,
        include_patterns: vec!["**/*.rs".to_string(), "**/*.py".to_string(), "**/*.js".to_string()],
        exclude_patterns: vec!["target/**".to_string(), "node_modules/**".to_string(), "**/*.pyc".to_string()],
        languages: vec!["rust".to_string(), "python".to_string(), "javascript".to_string(), "typescript".to_string()],
        enable_semantic_search: true,
        update_interval_seconds: 3600,
    })
}

#[utoipa::path(
    post,
    path = "/v1/index/config/validate",
    tag = "index",
    operation_id = "validate_config",
    request_body = IndexConfig,
    responses(
        (status = 200, description = "配置验证结果", body = ConfigValidationResponse),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn validate_config(Json(config): Json<IndexConfig>) -> Json<ConfigValidationResponse> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 验证最大文件大小
    if config.max_file_size == 0 {
        errors.push("max_file_size must be greater than 0".to_string());
    } else if config.max_file_size > 10240 {
        warnings.push("max_file_size is very large, may impact performance".to_string());
    }

    // 验证模式
    if config.include_patterns.is_empty() && config.exclude_patterns.is_empty() {
        warnings.push("No include or exclude patterns specified".to_string());
    }

    // 验证语言
    if config.languages.is_empty() {
        errors.push("At least one language must be specified".to_string());
    }

    Json(ConfigValidationResponse {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

#[utoipa::path(
    post,
    path = "/v1/index/analyze",
    tag = "index",
    operation_id = "analyze_code",
    request_body = AnalyzeRequest,
    responses(
        (status = 200, description = "代码分析结果", body = AnalyzeResponse),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn analyze_code(Json(request): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    // 简单的代码分析实现
    let lines_of_code = request.content.lines().count();
    let function_count = request.content.matches("fn ").count() +
                        request.content.matches("function ").count() +
                        request.content.matches("def ").count();
    let class_count = request.content.matches("class ").count() +
                     request.content.matches("struct ").count();

    // 简单的复杂度计算（基于控制流语句）
    let complexity = request.content.matches("if ").count() +
                    request.content.matches("for ").count() +
                    request.content.matches("while ").count() +
                    request.content.matches("match ").count() + 1;

    // 生成建议标签
    let mut suggested_tags = vec![request.language.clone()];
    if function_count > 0 {
        suggested_tags.push("functions".to_string());
    }
    if class_count > 0 {
        suggested_tags.push("classes".to_string());
    }

    // 简单的质量评分
    let quality_score = if lines_of_code > 0 {
        let comment_lines = request.content.lines().filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*")
        }).count();

        let comment_ratio = comment_lines as f32 / lines_of_code as f32;
        let base_score = 60;
        let comment_bonus = (comment_ratio * 40.0) as usize;
        let complexity_penalty = if complexity > 10 { (complexity - 10) * 2 } else { 0 };

        (base_score + comment_bonus).saturating_sub(complexity_penalty).min(100)
    } else {
        0
    };

    Json(AnalyzeResponse {
        complexity,
        function_count,
        class_count,
        lines_of_code,
        suggested_tags,
        quality_score,
    })
}

#[utoipa::path(
    get,
    path = "/v1/index/suggestions",
    tag = "index",
    operation_id = "get_suggestions",
    responses(
        (status = 200, description = "索引建议", body = Vec<IndexSuggestion>),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_suggestions() -> Json<Vec<IndexSuggestion>> {
    let suggestions = vec![
        IndexSuggestion {
            suggestion_type: "optimization".to_string(),
            description: "Consider adding more file types to improve search coverage".to_string(),
            priority: "medium".to_string(),
            related_index: None,
        },
        IndexSuggestion {
            suggestion_type: "maintenance".to_string(),
            description: "Index rebuild recommended for better performance".to_string(),
            priority: "low".to_string(),
            related_index: Some("idx_abc123def456".to_string()),
        },
    ];

    Json(suggestions)
}

#[utoipa::path(
    post,
    path = "/v1/index/batch/create",
    tag = "index",
    operation_id = "batch_create_index",
    request_body = BatchCreateRequest,
    responses(
        (status = 200, description = "批量创建状态", body = BatchStatus),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn batch_create_index(Json(request): Json<BatchCreateRequest>) -> Json<BatchStatus> {
    let batch_id = format!("batch-{}", Utc::now().timestamp());
    let total_tasks = request.sources.len();

    // TODO: 实现实际的批量创建逻辑

    Json(BatchStatus {
        batch_id,
        total_tasks,
        completed_tasks: 0,
        failed_tasks: 0,
        status: "started".to_string(),
        start_time: Utc::now().to_rfc3339(),
        estimated_completion: None,
    })
}

#[utoipa::path(
    get,
    path = "/v1/index/batch/{batch_id}/status",
    tag = "index",
    operation_id = "get_batch_status",
    params(
        ("batch_id" = String, Path, description = "批次ID"),
    ),
    responses(
        (status = 200, description = "批量操作状态", body = BatchStatus),
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_batch_status(Path(batch_id): Path<String>) -> Json<BatchStatus> {
    // TODO: 实现实际的批量状态查询逻辑
    Json(BatchStatus {
        batch_id,
        total_tasks: 5,
        completed_tasks: 3,
        failed_tasks: 1,
        status: "running".to_string(),
        start_time: Utc::now().to_rfc3339(),
        estimated_completion: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
    })
}

// 辅助函数
fn extract_search_result(doc: &TantivyDocument, score: f32, schema: &IndexSchema) -> Option<SearchResult> {
    let chunk_attributes = doc.get_first(schema.field_chunk_attributes)?;
    let attributes_text = chunk_attributes.as_str()?;
    let attributes: serde_json::Value = serde_json::from_str(attributes_text).ok()?;

    Some(SearchResult {
        file_path: attributes["filepath"].as_str()?.to_string(),
        content: attributes["body"].as_str()?.to_string(),
        start_line: attributes["start_line"].as_u64().map(|x| x as usize),
        end_line: None, // 可以根据内容计算
        score,
        language: attributes["language"].as_str()?.to_string(),
        git_url: attributes["git_url"].as_str()?.to_string(),
    })
}

fn extract_file_path(doc: &TantivyDocument, schema: &IndexSchema) -> Option<String> {
    let chunk_attributes = doc.get_first(schema.field_chunk_attributes)?;
    let attributes_text = chunk_attributes.as_str()?;
    let attributes: serde_json::Value = serde_json::from_str(attributes_text).ok()?;
    attributes["filepath"].as_str().map(|s| s.to_string())
}

fn to_json_value(doc: TantivyDocument, schema: &tantivy::schema::Schema) -> serde_json::Value {
    let json = doc.to_json(schema);
    let mut doc: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");

    for (_, value) in doc.as_object_mut().expect("Expected object").iter_mut() {
        if let Some(array) = value.as_array_mut() {
            if array.len() == 1 {
                *value = array[0].clone();
            }
        }
    }

    doc
}