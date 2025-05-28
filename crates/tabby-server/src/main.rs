use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tabby_db::DbConn;

mod api;

#[tokio::main]
async fn main() {
    // 初始化数据库连接
    let db = DbConn::new(std::path::Path::new("tabby.sqlite")).await.expect("数据库初始化失败");
    let db = Arc::new(db);

    // Build our application with a route
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/v1/index", api::index::router().with_state(db.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    // Run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}