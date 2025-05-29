use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tabby_common::axum::MaybeUser;
use tabby_schema::{
    ServiceLocator,
    model_configuration::{
        CreateAvailableModelInput, ModelType, PerformanceTier, UpdateAvailableModelInput,
        UpdateUserModelPreferenceInput,
    },
    DbEnum,
};
use juniper::ID;
use tracing::{info, warn};
use utoipa::ToSchema;
use crate::axum::extract::AuthBearer;

// === 用户模型偏好相关结构体 ===

#[derive(Serialize, ToSchema)]
#[schema(description = "用户模型偏好设置")]
pub struct UserModelPreferenceResponse {
    #[schema(example = "1")]
    pub user_id: String,

    #[schema(example = "gpt-3.5-turbo")]
    pub completion_model_id: Option<String>,

    #[schema(example = "gpt-4")]
    pub chat_model_id: Option<String>,

    #[schema(example = "2024-01-01T12:00:00Z")]
    pub created_at: String,

    #[schema(example = "2024-01-01T12:00:00Z")]
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
#[schema(description = "更新用户模型偏好的请求参数")]
pub struct UpdateUserModelPreferenceRequest {
    #[schema(example = "gpt-3.5-turbo")]
    pub completion_model_id: Option<String>,

    #[schema(example = "gpt-4")]
    pub chat_model_id: Option<String>,
}

// === 可用模型相关结构体 ===

#[derive(Serialize, ToSchema)]
#[schema(description = "可用模型信息")]
pub struct AvailableModelResponse {
    #[schema(example = "1")]
    pub id: String,

    #[schema(example = "gpt-3.5-turbo")]
    pub name: String,

    #[schema(example = "GPT-3.5 Turbo")]
    pub display_name: String,

    #[schema(example = "completion")]
    pub model_type: String,

    #[schema(example = "openai")]
    pub provider: String,

    #[schema(example = "balanced")]
    pub performance_tier: String,

    #[schema(example = true)]
    pub is_active: bool,

    #[schema(example = "2024-01-01T12:00:00Z")]
    pub created_at: String,

    #[schema(example = "2024-01-01T12:00:00Z")]
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
#[schema(description = "创建新模型的请求参数")]
pub struct CreateAvailableModelRequest {
    #[schema(example = "gpt-3.5-turbo")]
    pub name: String,

    #[schema(example = "GPT-3.5 Turbo")]
    pub display_name: String,

    #[schema(example = "completion")]
    pub model_type: String,

    #[schema(example = "openai")]
    pub provider: String,

    #[schema(example = "balanced")]
    pub performance_tier: String,

    #[schema(example = 4096)]
    pub max_tokens: Option<i32>,

    #[schema(example = 16384)]
    pub context_window: Option<i32>,

    #[schema(example = true)]
    pub enabled: Option<bool>,

    #[schema(example = "高效的代码补全模型")]
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[schema(description = "更新模型的请求参数")]
pub struct UpdateAvailableModelRequest {
    #[schema(example = "GPT-3.5 Turbo Updated")]
    pub display_name: Option<String>,

    #[schema(example = "quality")]
    pub performance_tier: Option<String>,

    #[schema(example = 8192)]
    pub max_tokens: Option<i32>,

    #[schema(example = 32768)]
    pub context_window: Option<i32>,

    #[schema(example = false)]
    pub enabled: Option<bool>,

    #[schema(example = "更新的模型描述")]
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[schema(description = "查询模型列表的参数")]
pub struct ListModelsQuery {
    #[schema(example = "completion")]
    pub model_type: Option<String>,
}

// === API端点实现 ===

#[utoipa::path(
    get,
    path = "/v1/user/model-preference",
    tag = "Model Configuration",
    operation_id = "get_user_model_preference",
    responses(
        (status = 200, description = "User model preference", body = UserModelPreferenceResponse),
        (status = 404, description = "User preference not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_user_model_preference(
    State(locator): State<Arc<dyn ServiceLocator>>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<UserModelPreferenceResponse>, StatusCode> {
    let user_id = get_user_id_from_token(token)?;

    match locator.model_configuration().get_user_model_preference(&user_id).await {
        Ok(Some(preference)) => Ok(Json(UserModelPreferenceResponse {
            user_id: preference.user_id.to_string(),
            completion_model_id: preference.completion_model_id.map(|id| id.to_string()),
            chat_model_id: preference.chat_model_id.map(|id| id.to_string()),
            created_at: preference.created_at.to_rfc3339(),
            updated_at: preference.updated_at.to_rfc3339(),
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to get user model preference: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    put,
    path = "/v1/user/model-preference",
    tag = "Model Configuration",
    operation_id = "update_user_model_preference",
    request_body = UpdateUserModelPreferenceRequest,
    responses(
        (status = 200, description = "Updated user model preference", body = UserModelPreferenceResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn update_user_model_preference(
    State(locator): State<Arc<dyn ServiceLocator>>,
    AuthBearer(token): AuthBearer,
    Json(request): Json<UpdateUserModelPreferenceRequest>,
) -> Result<Json<UserModelPreferenceResponse>, StatusCode> {
    let user_id = get_user_id_from_token(token)?;

    let input = UpdateUserModelPreferenceInput {
        completion_model_id: request.completion_model_id.map(|id| ID::from(id)),
        chat_model_id: request.chat_model_id.map(|id| ID::from(id)),
    };

    match locator.model_configuration().update_user_model_preference(&user_id, input).await {
        Ok(preference) => {
            info!("Updated model preference for user: {}", user_id);
            Ok(Json(UserModelPreferenceResponse {
                user_id: preference.user_id.to_string(),
                completion_model_id: preference.completion_model_id.map(|id| id.to_string()),
                chat_model_id: preference.chat_model_id.map(|id| id.to_string()),
                created_at: preference.created_at.to_rfc3339(),
                updated_at: preference.updated_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            warn!("Failed to update user model preference: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/models",
    tag = "Model Configuration",
    operation_id = "list_available_models",
    params(
        ("model_type" = Option<String>, Query, description = "Filter by model type (completion, chat)")
    ),
    responses(
        (status = 200, description = "List of available models", body = Vec<AvailableModelResponse>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn list_available_models(
    State(locator): State<Arc<dyn ServiceLocator>>,
    Query(query): Query<ListModelsQuery>,
) -> Result<Json<Vec<AvailableModelResponse>>, StatusCode> {
    let model_type = query.model_type.and_then(|t| {
        match t.as_str() {
            "completion" => Some(ModelType::Completion),
            "chat" => Some(ModelType::Chat),
            _ => None,
        }
    });

    match locator.model_configuration().list_available_models(model_type).await {
        Ok(models) => Ok(Json(models.into_iter().map(|model| AvailableModelResponse {
            id: model.id.to_string(),
            name: model.name,
            display_name: model.display_name,
            model_type: model.model_type.as_enum_str().to_string(),
            provider: model.provider,
            performance_tier: model.performance_tier.as_enum_str().to_string(),
            is_active: model.enabled,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }).collect())),
        Err(e) => {
            warn!("Failed to list available models: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/models/{id}",
    tag = "Model Configuration",
    operation_id = "get_available_model",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 200, description = "Available model", body = AvailableModelResponse),
        (status = 404, description = "Model not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_available_model(
    State(locator): State<Arc<dyn ServiceLocator>>,
    Path(id): Path<String>,
) -> Result<Json<AvailableModelResponse>, StatusCode> {
    let model_id = ID::from(id);

    match locator.model_configuration().get_available_model(&model_id).await {
        Ok(Some(model)) => Ok(Json(AvailableModelResponse {
            id: model.id.to_string(),
            name: model.name,
            display_name: model.display_name,
            model_type: model.model_type.as_enum_str().to_string(),
            provider: model.provider,
            performance_tier: model.performance_tier.as_enum_str().to_string(),
            is_active: model.enabled,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to get available model: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/models",
    tag = "Model Configuration",
    operation_id = "create_available_model",
    request_body = CreateAvailableModelRequest,
    responses(
        (status = 201, description = "Created model", body = AvailableModelResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn create_available_model(
    State(locator): State<Arc<dyn ServiceLocator>>,
    Json(request): Json<CreateAvailableModelRequest>,
) -> Result<Json<AvailableModelResponse>, StatusCode> {
    let model_type = match ModelType::from_enum_str(&request.model_type) {
        Ok(t) => t,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    let performance_tier = match PerformanceTier::from_enum_str(&request.performance_tier) {
        Ok(t) => t,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let input = CreateAvailableModelInput {
        name: request.name,
        display_name: request.display_name,
        model_type,
        provider: request.provider,
        performance_tier,
        max_tokens: request.max_tokens,
        context_window: request.context_window,
        enabled: request.enabled,
        description: request.description,
    };

    match locator.model_configuration().create_available_model(input).await {
        Ok(model) => {
            info!("Created new model: {}", model.name);
            Ok(Json(AvailableModelResponse {
                id: model.id.to_string(),
                name: model.name,
                display_name: model.display_name,
                model_type: model.model_type.as_enum_str().to_string(),
                provider: model.provider,
                performance_tier: model.performance_tier.as_enum_str().to_string(),
                is_active: model.enabled,
                created_at: model.created_at.to_rfc3339(),
                updated_at: model.updated_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            warn!("Failed to create available model: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    put,
    path = "/v1/models/{id}",
    tag = "Model Configuration",
    operation_id = "update_available_model",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    request_body = UpdateAvailableModelRequest,
    responses(
        (status = 200, description = "Updated model", body = AvailableModelResponse),
        (status = 400, description = "Invalid input"),
        (status = 404, description = "Model not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn update_available_model(
    State(locator): State<Arc<dyn ServiceLocator>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateAvailableModelRequest>,
) -> Result<Json<AvailableModelResponse>, StatusCode> {
    let model_id = ID::from(id);

    let performance_tier = if let Some(tier_str) = &request.performance_tier {
        match PerformanceTier::from_enum_str(tier_str) {
            Ok(tier) => Some(tier),
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    } else {
        None
    };

    let input = UpdateAvailableModelInput {
        display_name: request.display_name,
        performance_tier,
        max_tokens: request.max_tokens,
        context_window: request.context_window,
        enabled: request.enabled,
        description: request.description,
    };

    match locator.model_configuration().update_available_model(&model_id, input).await {
        Ok(model) => {
            info!("Updated model: {}", model.name);
            Ok(Json(AvailableModelResponse {
                id: model.id.to_string(),
                name: model.name,
                display_name: model.display_name,
                model_type: model.model_type.as_enum_str().to_string(),
                provider: model.provider,
                performance_tier: model.performance_tier.as_enum_str().to_string(),
                is_active: model.enabled,
                created_at: model.created_at.to_rfc3339(),
                updated_at: model.updated_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            warn!("Failed to update available model: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/v1/models/{id}",
    tag = "Model Configuration",
    operation_id = "delete_available_model",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 204, description = "Model deleted successfully"),
        (status = 404, description = "Model not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn delete_available_model(
    State(locator): State<Arc<dyn ServiceLocator>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let model_id = ID::from(id);

    match locator.model_configuration().delete_available_model(&model_id).await {
        Ok(_) => {
            info!("Deleted model: {}", model_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            warn!("Failed to delete available model: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// === 辅助函数 ===

fn get_user_id_from_token(token: Option<String>) -> Result<ID, StatusCode> {
    match token {
        Some(token) => {
            match crate::jwt::validate_jwt(&token) {
                Ok(claims) => Ok(ID::from(claims.sub)),
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}