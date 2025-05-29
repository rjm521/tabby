use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use axum_extra::TypedHeader;
use tabby_common::axum::MaybeUser;
use tabby_schema::{
    AsID, AsRowid, ServiceLocator,
};
use tracing::{info, warn};
use juniper::ID;
use utoipa::ToSchema;

// 简化的聊天请求和响应类型
#[derive(serde::Deserialize, ToSchema)]
pub struct ChatRequest {
    /// Array of chat messages
    pub messages: Vec<ChatMessage>,
    /// Optional specific model to use for chat
    pub model: Option<String>,
    /// Whether to stream the response
    pub stream: Option<bool>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct ChatMessage {
    /// Message role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ChatResponse {
    /// Response message from EE chat service
    pub message: String,
    /// Model used for chat completion
    pub model_used: String,
}

#[utoipa::path(
    post,
    path = "/v1/ee/chat/completions",
    tag = "v1",
    operation_id = "ee_chat_completions",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "EE Chat completion response", body = ChatResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn ee_chat_completions(
    State(locator): State<Arc<dyn ServiceLocator>>,
    TypedHeader(MaybeUser(user_jwt_sub)): TypedHeader<MaybeUser>,
    Json(request): Json<ChatRequest>,
) -> Result<axum::Json<ChatResponse>, StatusCode> {
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
                if let Some(_preferred_model_id) = prefs.chat_model_id {
                    // 获取用户首选的聊天模型
                    match locator.model_configuration().get_user_chat_model(&user_id).await {
                        Ok(Some(model)) => {
                            info!("User {} using preferred chat model: {}", uid, model.name);
                            final_model_name = Some(model.name);
                        }
                        Ok(None) => {
                            warn!("User {} preferred chat model not found. Using default.", uid);
                        }
                        Err(e) => {
                            warn!("Failed to get chat model for user {}: {:?}", uid, e);
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

    let model_used = final_model_name
        .or(request.model)
        .unwrap_or_else(|| "default".to_string());

    // 构建简单的聊天响应
    let last_message = request.messages.last()
        .map(|m| m.content.as_str())
        .unwrap_or("Hello");

    Ok(axum::Json(ChatResponse {
        message: format!("EE Chat response to: '{}' using model: {}", last_message, model_used),
        model_used,
    }))
}