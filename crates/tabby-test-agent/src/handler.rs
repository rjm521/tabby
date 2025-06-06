use axum::{Json};
use serde::{Deserialize, Serialize};
use crate::ai;
use axum::http::StatusCode;
// 引入 AuthBearer 提取器

// 你需要将 AuthBearer 结构体和 get_user_id_from_token 函数复制到本 crate 或通过依赖引入
#[derive(Deserialize)]
pub struct TestCaseRequest {
    pub api_desc: String,
}

#[derive(Serialize)]
pub struct TestCaseResponse {
    pub test_case: String,
}

// ====== JWT Bearer 提取器实现 ======
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthBearer(pub Option<String>);

type Rejection = (StatusCode, &'static str);

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthBearer
where
    S: Send + Sync,
{
    type Rejection = Rejection;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get("authorization")
            .and_then(|hv| hv.to_str().ok());
        let token = authorization
            .and_then(|auth| auth.strip_prefix("Bearer "))
            .map(|s| s.to_string());
        Ok(AuthBearer(token))
    }
}

// ====== JWT 校验真实实现 ======
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::Deserialize as SerdeDeserialize;

#[derive(Debug, SerdeDeserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

fn get_jwt_secret() -> String {
    std::env::var("TABBY_WEBSERVER_JWT_TOKEN_SECRET")
        .expect("请设置环境变量 TABBY_WEBSERVER_JWT_TOKEN_SECRET")
}

fn get_user_id_from_token(token: Option<String>) -> Result<String, StatusCode> {
    match token {
        Some(token) => {
            let secret = get_jwt_secret();
            let validation = Validation::new(Algorithm::HS256);
            let token_data = decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
            Ok(token_data.claims.sub)
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// 生成接口测试用例的 HTTP 处理函数，临时关闭JWT鉴权（仅用于测试）
pub async fn generate_test_case(
    // AuthBearer(token): AuthBearer, // 注释掉token校验
    Json(payload): Json<TestCaseRequest>,
) -> Result<Json<TestCaseResponse>, StatusCode> {
    // let user_id = get_user_id_from_token(token)?;
    // 你可以用 user_id 做用户隔离
    let test_case = ai::generate_test_case_with_rig(&payload.api_desc).await;
    Ok(Json(TestCaseResponse { test_case }))
}