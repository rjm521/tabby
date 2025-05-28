use std::{path::PathBuf, sync::Arc};
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use git2::Repository;
use tempfile::TempDir;
use walkdir::WalkDir;
use glob::Pattern;
use tantivy::{
    schema::{Schema, SchemaBuilder, TEXT, STORED},
    Index,
    doc,
};
use tabby_db::DbConn;

#[derive(Debug, Deserialize)]
pub struct CreateIndexRequest {
    pub source: String,
    pub name: String,
    pub language: String,
    pub max_file_size: usize,
    pub exclude: Vec<String>,
    pub include: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateIndexResponse {
    pub status: String,
    pub message: String,
}

pub fn router() -> Router<Arc<DbConn>> {
    Router::new()
        .route("/create", post(create_index))
        .route("/user/register", post(super::user::register_user))
        .route("/user/token", get(super::user::query_user_token))
}

fn create_schema() -> Schema {
    let mut schema_builder = SchemaBuilder::default();
    schema_builder.add_text_field("path", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT);
    schema_builder.add_text_field("language", TEXT | STORED);
    schema_builder.build()
}

async fn create_index(
    Json(payload): Json<CreateIndexRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<CreateIndexResponse>)> {
    // Create a temporary directory for cloning
    let temp_dir = TempDir::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreateIndexResponse {
                status: "error".to_string(),
                message: format!("Failed to create temporary directory: {}", e),
            }),
        )
    })?;

    // Clone the repository
    let _repo = match Repository::clone(&payload.source, temp_dir.path()) {
        Ok(repo) => repo,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(CreateIndexResponse {
                    status: "error".to_string(),
                    message: format!("Failed to clone repository: {}", e),
                }),
            ))
        }
    };

    // Create index directory
    let index_dir = PathBuf::from("indices").join(&payload.name);
    fs::create_dir_all(&index_dir).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreateIndexResponse {
                status: "error".to_string(),
                message: format!("Failed to create index directory: {}", e),
            }),
        )
    })?;

    // Create schema and index
    let schema = create_schema();
    let index = Index::create_in_dir(&index_dir, schema.clone()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreateIndexResponse {
                status: "error".to_string(),
                message: format!("Failed to create index: {}", e),
            }),
        )
    })?;

    let mut index_writer = index.writer(50_000_000).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreateIndexResponse {
                status: "error".to_string(),
                message: format!("Failed to create index writer: {}", e),
            }),
        )
    })?;

    // Compile glob patterns
    let exclude_patterns: Vec<Pattern> = payload.exclude
        .iter()
        .map(|p| Pattern::new(p).unwrap())
        .collect();
    let include_patterns: Vec<Pattern> = payload.include
        .iter()
        .map(|p| Pattern::new(p).unwrap())
        .collect();

    // Walk through repository files
    for entry in WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative_path = path.strip_prefix(temp_dir.path()).unwrap();
        let path_str = relative_path.to_string_lossy();

        // Check if file matches include/exclude patterns
        if !include_patterns.is_empty() && !include_patterns.iter().any(|p| p.matches(&path_str)) {
            continue;
        }
        if exclude_patterns.iter().any(|p| p.matches(&path_str)) {
            continue;
        }

        // Read file content
        let content = match fs::read_to_string(path).await {
            Ok(content) => content,
            Err(_) => continue,
        };

        // Skip if file is too large
        if content.len() > payload.max_file_size * 1024 {
            continue;
        }

        // Add document to index
        let doc = doc!(
            schema.get_field("path").unwrap() => path_str.to_string(),
            schema.get_field("content").unwrap() => content,
            schema.get_field("language").unwrap() => payload.language.clone(),
        );

        index_writer.add_document(doc).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CreateIndexResponse {
                    status: "error".to_string(),
                    message: format!("Failed to add document to index: {}", e),
                }),
            )
        })?;
    }

    // Commit the index
    index_writer.commit().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreateIndexResponse {
                status: "error".to_string(),
                message: format!("Failed to commit index: {}", e),
            }),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(CreateIndexResponse {
            status: "success".to_string(),
            message: format!("Successfully created index for {}", payload.name),
        }),
    ))
}