use axum::{
    extract::Path,
    response::sse::{Event, Sse},
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use futures::stream::{self, Stream};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tabby_index;
use crate::services::embedding;
use tabby_common::config::CodeRepository;
use std::io::Cursor;
use zip::ZipArchive;
use tantivy::{
    DocAddress, DocSet,
    schema::Term,
    Index,
    TERMINATED, TantivyDocument, Document,
};
use tabby_common::index::IndexSchema;
use chrono::{Duration, Utc};

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
    /// 可选的消息
    #[schema(example = "Index created successfully")]
    message: Option<String>,
}

pub fn index_router() -> Router {
    Router::new()
        .route("/v1/index/info", get(get_index_info))
        .route("/v1/index/documents/{corpus}", get(get_documents))
        .route("/v1/index/create", post(create_index))
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

#[derive(Deserialize, ToSchema)]
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

#[derive(Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexingProgress {
    /// 总文件数
    total_files: usize,
    /// 已处理文件数
    processed_files: usize,
    /// 已更新chunks数
    updated_chunks: usize,
    /// 进度百分比
    progress_percentage: f32,
    /// 当前状态
    status: String,
    /// 当前正在处理的文件路径
    current_file: Option<String>,
    /// 索引开始时间
    start_time: Option<String>,
    /// 预估完成时间
    estimated_completion: Option<String>,
    /// 处理速度 (文件/秒)
    processing_rate: Option<f32>,
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

    tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        let start_time_str = chrono::Utc::now().to_rfc3339();

        let mut progress = IndexingProgress {
            total_files: 0,
            processed_files: 0,
            updated_chunks: 0,
            progress_percentage: 0.0,
            status: "准备中...".to_string(),
            current_file: None,
            start_time: Some(start_time_str.clone()),
            estimated_completion: None,
            processing_rate: None,
        };

        // 发送初始状态
        let _ = tx.send(progress.clone()).await;

        let repository = if request.is_remote_zip.unwrap_or(false) {
            // 下载并解压远程zip文件
            progress.status = "下载远程文件...".to_string();
            progress.current_file = Some(request.source.clone());
            let _ = tx.send(progress.clone()).await;

            match reqwest::get(&request.source).await {
                Ok(response) => {
                    progress.status = "获取文件内容...".to_string();
                    let _ = tx.send(progress.clone()).await;

                    match response.bytes().await {
                        Ok(bytes) => {
                            progress.status = "解压文件...".to_string();
                            let _ = tx.send(progress.clone()).await;

                            let cursor = Cursor::new(bytes);
                            match ZipArchive::new(cursor) {
                                Ok(mut archive) => {
                                    let temp_dir = tempfile::tempdir().unwrap();
                                    match archive.extract(&temp_dir) {
                                        Ok(_) => {
                                            CodeRepository {
                                                git_url: temp_dir.path().to_string_lossy().to_string(),
                                                source_id: request.name.unwrap_or_else(|| "default".to_string()),
                                            }
                                        }
                                        Err(e) => {
                                            progress.status = format!("解压失败: {}", e);
                                            let _ = tx.send(progress).await;
                                            return;
                                        }
                                    }
                                }
                                Err(e) => {
                                    progress.status = format!("ZIP文件格式错误: {}", e);
                                    let _ = tx.send(progress).await;
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            progress.status = format!("下载失败: {}", e);
                            let _ = tx.send(progress).await;
                            return;
                        }
                    }
                }
                Err(e) => {
                    progress.status = format!("网络请求失败: {}", e);
                    let _ = tx.send(progress).await;
                    return;
                }
            }
        } else {
            CodeRepository {
                git_url: request.source.clone(),
                source_id: request.name.unwrap_or_else(|| "default".to_string()),
            }
        };

        // 获取 embedding 配置
        progress.status = "初始化embedding模型...".to_string();
        progress.current_file = None;
        let _ = tx.send(progress.clone()).await;

        let config = match tabby_common::config::Config::load() {
            Ok(config) => config.model.embedding,
            Err(e) => {
                progress.status = format!("配置加载失败: {}", e);
                let _ = tx.send(progress).await;
                return;
            }
        };

        let embedding = embedding::create(&config).await;
        let mut indexer = tabby_index::public::CodeIndexer::default();

        // 设置进度回调
        let tx_clone = tx.clone();
        let start_time_clone = start_time;
        let start_time_str_clone = start_time_str.clone();
        indexer.set_progress_callback(Box::new(move |total_files, processed_files, updated_chunks| {
            let elapsed = start_time_clone.elapsed().as_secs_f32();
            let progress_percentage = if total_files > 0 {
                (processed_files as f32 / total_files as f32) * 100.0
            } else {
                0.0
            };

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

            let status = if processed_files == 0 {
                "扫描文件中...".to_string()
            } else if processed_files < total_files {
                format!("正在索引文件... ({}/{})", processed_files, total_files)
            } else {
                "完成文件处理".to_string()
            };

            let progress = IndexingProgress {
                total_files,
                processed_files,
                updated_chunks,
                progress_percentage,
                status,
                current_file: None,
                start_time: Some(start_time_str_clone.clone()),
                estimated_completion,
                processing_rate: Some(processing_rate),
            };

            let _ = tx_clone.try_send(progress);
        }));

        // 开始索引
        progress.status = "开始建立索引...".to_string();
        progress.progress_percentage = 0.0;
        let _ = tx.send(progress.clone()).await;

        match indexer.refresh(embedding, &repository).await {
            Ok(_) => {
                let elapsed = start_time.elapsed().as_secs_f32();
                progress.status = "索引创建完成".to_string();
                progress.progress_percentage = 100.0;
                progress.current_file = None;
                progress.estimated_completion = None;
                progress.processing_rate = Some(progress.processed_files as f32 / elapsed.max(1.0));
                let _ = tx.send(progress).await;
            }
            Err(e) => {
                progress.status = format!("索引创建失败: {}", e);
                progress.current_file = None;
                progress.estimated_completion = None;
                let _ = tx.send(progress).await;
            }
        }
    });

    let stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Some(progress) => {
                let event = Event::default().json_data(progress).unwrap();
                Some((Ok(event), rx))
            }
            None => None,
        }
    });

    Sse::new(stream)
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