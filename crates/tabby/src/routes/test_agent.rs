use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use crate::services::test_agent::TestAgentService;

#[derive(Deserialize)]
pub struct TestCaseRequest {
    pub api_desc: String,
}

#[derive(Serialize)]
pub struct TestCaseResponse {
    pub test_case: String,
}

pub fn router(test_agent: TestAgentService) -> Router {
    Router::new()
        .route("/v1/test/generate", post(generate_test_case))
        .with_state(test_agent)
}

#[axum::debug_handler]
async fn generate_test_case(
    State(test_agent): State<TestAgentService>,
    Json(payload): Json<TestCaseRequest>,
) -> Result<Json<TestCaseResponse>, StatusCode> {
    let test_case = test_agent.generate_test_case(&payload.api_desc).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(TestCaseResponse { test_case }))
}