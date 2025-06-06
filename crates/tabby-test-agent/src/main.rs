mod handler;
mod ai;
mod config;
use tokio::net::TcpListener;
use axum::serve;

#[tokio::main]
async fn main() {
    // 注册路由
    let app = axum::Router::new()
        .route("/v1/test/generate", axum::routing::post(handler::generate_test_case));

    // 启动服务（修改端口到8081避免冲突）
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("tabby-test-agent 服务已启动，监听 0.0.0.0:8081");
    serve(listener, app).await.unwrap();
}