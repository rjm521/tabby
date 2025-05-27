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
        let mut progress = IndexingProgress {
            total_files: 0,
            processed_files: 0,
            updated_chunks: 0,
            progress_percentage: 0.0,
            status: "准备中...".to_string(),
        };

        // 发送初始状态
        let _ = tx.send(progress.clone()).await;

        let repository = if request.is_remote_zip.unwrap_or(false) {
            // 下载并解压远程zip文件
            progress.status = "下载远程文件...".to_string();
            let _ = tx.send(progress.clone()).await;

            let response = reqwest::get(&request.source).await.unwrap();
            let bytes = response.bytes().await.unwrap();

            progress.status = "解压文件...".to_string();
            let _ = tx.send(progress.clone()).await;

            let cursor = Cursor::new(bytes);
            let mut archive = ZipArchive::new(cursor).unwrap();

            let temp_dir = tempfile::tempdir().unwrap();
            archive.extract(&temp_dir).unwrap();

            CodeRepository {
                git_url: temp_dir.path().to_string_lossy().to_string(),
                source_id: request.name.unwrap_or_else(|| "default".to_string()),
            }
        } else {
            CodeRepository {
                git_url: request.source.clone(),
                source_id: request.name.unwrap_or_else(|| "default".to_string()),
            }
        };

        // 获取 embedding 配置
        let config = tabby_common::config::Config::load().unwrap().model.embedding;
        let embedding = embedding::create(&config).await;
        let mut indexer = tabby_index::public::CodeIndexer::default();

        // 开始索引
        progress.status = "正在索引...".to_string();
        progress.progress_percentage = 50.0;
        let _ = tx.send(progress.clone()).await;

        match indexer.refresh(embedding, &repository).await {
            Ok(_) => {
                progress.status = "索引完成".to_string();
                progress.progress_percentage = 100.0;
                let _ = tx.send(progress).await;
            }
            Err(e) => {
                progress.status = format!("索引失败: {}", e);
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