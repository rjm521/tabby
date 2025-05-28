mod hub;
pub mod ingestion;
mod oauth;
mod repositories;
mod ui;
pub mod ee_completions;
pub mod ee_chat;

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::{from_fn_with_state, Next},
    response::{IntoResponse, Response},
    routing, Extension, Json, Router,
};
use juniper::ID;
use juniper_axum::{graphiql, playground};
use serde::{Deserialize, Serialize};
use tabby_common::api::server_setting::ServerSetting;
use tabby_schema::{
    auth::AuthenticationService, create_schema, job::JobService, Schema, ServiceLocator,
};
use tower::util::ServiceExt;
use tower_http::services::ServeFile;
use tracing::{error, warn};
use utoipa::ToSchema;

use self::hub::HubState;
use crate::{
    axum::{extract::AuthBearer, graphql, FromAuth},
    jwt::{generate_jwt_payload, validate_jwt, generate_jwt},
    service::answer::AnswerService,
};

// Assuming StandardCompletionService is from tabby crate, aliased to avoid conflict if needed
// use tabby::services::completion::CompletionService as StandardCompletionService;
// use tabby::routes::chat::ChatState as StandardChatState;

#[derive(Deserialize, ToSchema)]
pub struct GetTokenRequest {
    #[schema(example = "user@example.com")]
    email: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetTokenResponse {
    #[serde(rename = "accessToken")]
    #[schema(example = "your_access_token_here")]
    access_token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GraphqlHttpRequest {
    #[schema(example = "query { me { email } }")]
    query: String,
    #[schema(example = json!({ "variableName": "value" }))]
    variables: Option<serde_json::Value>,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "新用户")]
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    #[serde(rename = "accessToken")]
    #[schema(example = "your_access_token_here")]
    pub access_token: String,
}

pub fn create(
    ctx: Arc<dyn ServiceLocator>,
    mut api: Router,
    ui: Router,
    _answer: Option<Arc<AnswerService>>,
) -> (Router, Router) {
    let schema = Arc::new(create_schema());

    let protected_api = Router::new()
        .route(
            "/background-jobs/{id}/logs",
            routing::get(background_job_logs).with_state(ctx.job()),
        )
        // Add other endpoints that need authentication here
        .layer(from_fn_with_state(ctx.auth(), require_login_middleware));

    // Ingestion APIs are protected by registration token
    let registration_api = Router::new()
        .route(
            "/v1beta/ingestion",
            routing::post(ingestion::ingestion).with_state(Arc::new(ingestion::IngestionState {
                ingestion: ctx.ingestion(),
            })),
        )
        .route(
            "/v1beta/ingestion/{source}",
            routing::delete(ingestion::delete_ingestion_source).with_state(Arc::new(
                ingestion::IngestionState {
                    ingestion: ctx.ingestion(),
                },
            )),
        )
        .route(
            "/v1beta/ingestion/{source}/{id}",
            routing::delete(ingestion::delete_ingestion).with_state(Arc::new(
                ingestion::IngestionState {
                    ingestion: ctx.ingestion(),
                },
            )),
        )
        .layer(from_fn_with_state(ctx.clone(), require_registration_token));

    api = api.route(
        "/v1beta/server_setting",
        routing::get(server_setting).with_state(ctx.clone()),
    );

    // EE version overrides/adds the /v1/completions route
    // It needs the main ServiceLocator (ctx) and the standard CompletionService as an extension.
    // The standard CompletionService should be added as an Extension in Webserver::attach or where `api` router is built.
    api = api.route(
        "/v1/completions",
        routing::post(ee_completions::ee_completions).with_state(ctx.clone())
    );

    // EE chat completions route
    // Assumes StandardChatState is available as an Extension layer on `api` router
    api = api.route(
        "/v1/chat/completions",
        routing::post(ee_chat::ee_chat_completions).with_state(ctx.clone())
    );

    api = api
        // Routes before `distributed_tabby_layer` are protected by authentication middleware for following routes:
        // 1. /v1/*
        // 2. /v1beta/*
        .layer(from_fn_with_state(ctx.clone(), distributed_tabby_layer))
        .merge(protected_api)
        .merge(registration_api)
        .route(
            "/graphql",
            routing::post(graphql::<Arc<Schema>, Arc<dyn ServiceLocator>>).with_state(ctx.clone()),
        )
        .route(
            "/subscriptions",
            routing::get(crate::axum::subscriptions::<Arc<Schema>, Arc<dyn ServiceLocator>>)
                .with_state(ctx.clone()),
        )
        .route(
            "/graphql",
            routing::get(playground("/graphql", "/subscriptions")),
        )
        .layer(Extension(schema))
        .route(
            "/hub",
            routing::get(hub::ws_handler).with_state(HubState::new(ctx.clone()).into()),
        )
        .nest(
            "/repositories",
            repositories::routes(ctx.repository(), ctx.auth()),
        )
        .route("/avatar/{id}", routing::get(avatar).with_state(ctx.auth()))
        .nest("/oauth", oauth::routes(ctx.auth()))
        .route("/v1/auth/token", routing::post(get_user_token).with_state(ctx.auth()))
        // Add the new route for executing GraphQL over HTTP
        .route("/v1/graphql", routing::post(execute_graphql_http).with_state(ctx.clone()))
        // Add the new RESTful register API
        .route("/v1/auth/register", routing::post(register_user).with_state(ctx.auth()));

    let ui = ui.route(
        "/graphiql",
        routing::get(graphiql("/graphql", "/subscriptions")),
    );

    let ui = ui.fallback(ui::handler);

    (api, ui)
}

pub(crate) async fn require_login_middleware(
    State(auth): State<Arc<dyn AuthenticationService>>,
    AuthBearer(token): AuthBearer,
    mut request: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let unauthorized = axum::response::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::empty())
        .unwrap()
        .into_response();

    let Some(token) = token else {
        return unauthorized;
    };

    let Ok(jwt) = auth.verify_access_token(&token).await else {
        return unauthorized;
    };

    let Ok(user) = auth.get_user(&jwt.sub).await else {
        return unauthorized;
    };

    request.extensions_mut().insert(user.policy);

    next.run(request).await
}

async fn distributed_tabby_layer(
    State(ws): State<Arc<dyn ServiceLocator>>,
    request: Request<Body>,
    next: Next,
) -> axum::response::Response {
    ws.worker().dispatch_request(request, next).await
}

pub(crate) async fn require_registration_token(
    State(locator): State<Arc<dyn ServiceLocator>>,
    AuthBearer(token): AuthBearer,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let unauthorized = axum::response::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::empty())
        .unwrap()
        .into_response();

    let Some(token) = token else {
        return unauthorized;
    };

    let Ok(registration_token) = locator.worker().read_registration_token().await else {
        return unauthorized;
    };

    if token != registration_token {
        return unauthorized;
    }

    next.run(request).await
}

async fn server_setting(
    State(locator): State<Arc<dyn ServiceLocator>>,
) -> Result<Json<ServerSetting>, StatusCode> {
    let security_setting = match locator.setting().read_security_setting().await {
        Ok(x) => x,
        Err(err) => {
            warn!("Failed to read security setting {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(ServerSetting {
        disable_client_side_telemetry: security_setting.disable_client_side_telemetry,
    }))
}

async fn avatar(
    State(state): State<Arc<dyn AuthenticationService>>,
    Path(id): Path<ID>,
) -> Result<Response<Body>, StatusCode> {
    let avatar = state
        .get_user_avatar(&id)
        .await
        .map_err(|e| {
            error!("Failed to retrieve avatar: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    let mut response = Response::new(Body::from(avatar.into_vec()));
    response
        .headers_mut()
        .insert("Content-Type", "image/*".parse().unwrap());
    Ok(response)
}

// Handler for the new /v1/auth/token API
#[utoipa::path(
    post,
    path = "/v1/auth/token",
    request_body = GetTokenRequest,
    responses(
        (status = 200, description = "Successfully retrieved token", body = GetTokenResponse),
        (status = 400, description = "Invalid input"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_user_token(
    State(auth_service): State<Arc<dyn AuthenticationService>>,
    Json(payload): Json<GetTokenRequest>,
) -> Result<Json<GetTokenResponse>, StatusCode> {
    if payload.email.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match auth_service.get_user_by_email(&payload.email).await {
        Ok(user) => {
            // Directly generate a new access token for the user without password validation
            match generate_jwt(user.id) { // Assuming user.id is compatible with generate_jwt
                Ok(access_token) => Ok(Json(GetTokenResponse { access_token })),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR), // Failed to generate token
            }
        }
        Err(_) => Err(StatusCode::NOT_FOUND), // User not found by email
    }
}

// Handler for the new /v1/graphql API
#[utoipa::path(
    post,
    path = "/v1/graphql",
    request_body = GraphqlHttpRequest,
    responses(
        (status = 200, description = "GraphQL query executed successfully", body = serde_json::Value),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error or GraphQL execution error")
    ),
)]
pub async fn execute_graphql_http(
    State(locator): State<Arc<dyn ServiceLocator>>,
    Extension(schema): Extension<Arc<Schema>>,
    AuthBearer(token): AuthBearer, // To pass along authentication info if present
    Json(payload): Json<GraphqlHttpRequest>,
) -> impl IntoResponse {
    let context = tabby_schema::Context::build(locator.clone(), token).await;

    // Convert serde_json::Value to juniper::Variables
    let vars = match payload.variables {
        Some(v) => match serde_json::from_value::<juniper::Variables>(v) {
            Ok(vars) => vars,
            Err(err) => {
                warn!("Failed to deserialize GraphQL variables: {}", err);
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        None => juniper::Variables::new(),
    };

    let execution_result = juniper::execute(
        &payload.query,
        None, // operation_name
        &schema,
        &vars,
        &context,
    )
    .await;

    // Serialize execution result to JSON
    let body_json = match serde_json::to_string(&execution_result) {
        Ok(s) => s,
        Err(err) => {
            warn!("Failed to serialize GraphQL execution result: {}", err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body_json))
        .unwrap()
}

#[async_trait::async_trait]
impl FromAuth<Arc<dyn ServiceLocator>> for tabby_schema::Context {
    async fn build(locator: Arc<dyn ServiceLocator>, token: Option<String>) -> Self {
        let claims = if let Some(token) = token {
            let mut claims = validate_jwt(&token).ok();

            if claims.is_none() {
                claims = locator
                    .auth()
                    .verify_auth_token(&token)
                    .await
                    .ok()
                    .map(|id| generate_jwt_payload(id, true));
            }

            claims
        } else {
            None
        };

        Self { claims, locator }
    }
}

async fn background_job_logs(
    State(state): State<Arc<dyn JobService>>,
    Path(id): Path<ID>,
    request: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let log_file_path = state
        .log_file_path(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let serve_file = ServeFile::new(log_file_path);
    match serve_file.oneshot(request).await {
        Ok(response) => Ok(response.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "注册成功", body = RegisterResponse),
        (status = 400, description = "参数错误"),
        (status = 409, description = "用户已存在"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn register_user(
    State(auth_service): State<Arc<dyn AuthenticationService>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    if payload.email.is_empty() || payload.name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    // 检查用户是否已存在
    if auth_service.get_user_by_email(&payload.email).await.is_ok() {
        return Err(StatusCode::CONFLICT);
    }
    // 默认密码
    let password = "TabbyR0cks!".to_string();
    // 调用注册逻辑
    match auth_service.register(payload.email.clone(), password, None, Some(payload.name.clone())).await {
        Ok(_) => {
            // 注册成功后直接生成token
            match auth_service.get_user_by_email(&payload.email).await {
                Ok(user) => match generate_jwt(user.id) {
                    Ok(access_token) => Ok(Json(RegisterResponse { access_token })),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                },
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
