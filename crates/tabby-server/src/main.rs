use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

mod api;

#[tokio::main]
async fn main() {
    // Build our application with a route
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/v1/index", api::index::router())
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