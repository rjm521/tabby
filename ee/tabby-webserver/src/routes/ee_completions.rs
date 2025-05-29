use std::sync::Arc;

use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use axum_extra::{
    headers::UserAgent,
    TypedHeader,
};
use tabby_common::axum::{AllowedCodeRepository, MaybeUser};
use tabby_schema::{
    AsID, AsRowid, ServiceLocator,
};
use tracing::{info, warn};
use juniper::ID;
use utoipa::ToSchema;

// 为了简化，我们暂时返回一个简单的响应类型
#[derive(serde::Serialize, ToSchema)]
pub struct CompletionResponse {
    /// Response message from EE completion service
    pub message: String,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct CompletionRequest {
    /// The prompt for code completion
    pub prompt: String,
    /// Optional specific model to use for completion
    pub model: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/ee/completions",
    tag = "v1",
    operation_id = "ee_completions",
    request_body = CompletionRequest,
    responses(
        (status = 200, description = "EE Completion response", body = CompletionResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn ee_completions(
    State(locator): State<Arc<dyn ServiceLocator>>,
    Extension(_allowed_code_repository): Extension<AllowedCodeRepository>,
    TypedHeader(MaybeUser(user_jwt_sub)): TypedHeader<MaybeUser>,
    _user_agent: Option<TypedHeader<UserAgent>>,
    Json(request): Json<CompletionRequest>,
) -> Result<Json<CompletionResponse>, StatusCode> {
    let mut user_id_for_preference: Option<i64> = None;

    if let Some(sub) = &user_jwt_sub {
        // Convert sub string to ID using ID::from()
        let user_id = ID::from(sub.clone());
        match locator.auth().get_user(&user_id).await {
            Ok(user_secured) => match user_secured.id.as_rowid() {
                Ok(id) => user_id_for_preference = Some(id),
                Err(_) => {
                    warn!("Failed to convert user ID to rowid for sub: {}", sub);
                }
            },
            Err(e) => {
                warn!("Failed to retrieve user for preference from sub {}: {:?}", sub, e);
            }
        }
    }

    let mut final_model_name: Option<String> = None;
    if let Some(uid) = user_id_for_preference {
        let user_id = uid.as_id();
        match locator.model_configuration().get_user_model_preference(&user_id).await {
            Ok(Some(prefs)) => {
                if let Some(_preferred_model_id) = prefs.completion_model_id {
                    // 获取用户首选的补全模型
                    match locator.model_configuration().get_user_completion_model(&user_id).await {
                        Ok(Some(model)) => {
                            info!("User {} using preferred completion model: {}", uid, model.name);
                            final_model_name = Some(model.name);
                        }
                        Ok(None) => {
                            warn!("User {} preferred completion model not found. Using default.", uid);
                        }
                        Err(e) => {
                            warn!("Failed to get completion model for user {}: {:?}", uid, e);
                        }
                    }
                }
            }
            Ok(None) => { /* No preference set, use default */ }
            Err(e) => {
                warn!("Failed to get model preference for user {}: {:?}", uid, e);
            }
        }
    }

    // 返回简化的响应
    Ok(Json(CompletionResponse {
        message: format!(
            "EE completion request processed. Prompt: '{}', Model: {:?}",
            request.prompt,
            final_model_name.or(request.model).unwrap_or_else(|| "default".to_string())
        ),
    }))
}