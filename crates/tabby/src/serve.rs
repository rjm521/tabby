use std::{net::IpAddr, sync::Arc, time::Duration};

use axum::{routing, Extension, Router};
use clap::Args;
use hyper::StatusCode;
use spinners::{Spinner, Spinners, Stream};
use tabby_common::{
    api::{self, code::CodeSearch, event::EventLogger},
    axum::AllowedCodeRepository,
    config::{Config, ModelConfig},
    usage,
};
use tabby_download::ModelKind;
#[cfg(feature = "ee")]
use tabby_webserver::EEApiDoc;
use tokio::{sync::oneshot::Sender, time::sleep};
use tower_http::timeout::TimeoutLayer;
use tracing::{debug, warn};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    routes::{self, run_app, ChatState, test_agent},
    services::{
        self,
        code::create_code_search,
        completion::{self, create_completion_service_and_chat, CompletionService},
        embedding,
        event::create_event_logger,
        health,
        model::download_model_if_needed,
        tantivy::IndexReaderProvider,
        test_agent::TestAgentService,
    },
    to_local_config, Device,
};

#[derive(OpenApi)]
#[openapi(
    info(title="Tabby Server",
        description = "
[![tabby stars](https://img.shields.io/github/stars/TabbyML/tabby)](https://github.com/TabbyML/tabby)
[![Join Slack](https://shields.io/badge/Join-Tabby%20Slack-red?logo=slack)](https://links.tabbyml.com/join-slack)

Install following IDE / Editor extensions to get started with [Tabby](https://github.com/TabbyML/tabby).
* [VSCode Extension](https://github.com/TabbyML/tabby/tree/main/clients/vscode) – Install from the [marketplace](https://marketplace.visualstudio.com/items?itemName=TabbyML.vscode-tabby), or [open-vsx.org](https://open-vsx.org/extension/TabbyML/vscode-tabby)
* [VIM Extension](https://github.com/TabbyML/tabby/tree/main/clients/vim)
* [IntelliJ Platform Plugin](https://github.com/TabbyML/tabby/tree/main/clients/intellij) – Install from the [marketplace](https://plugins.jetbrains.com/plugin/22379-tabby)
",
        license(name = "Apache 2.0", url="https://github.com/TabbyML/tabby/blob/main/LICENSE")
    ),
    servers(
        (url = "/", description = "Server"),
    ),
    paths(
        routes::log_event,
        routes::completions,
        routes::chat_completions_utoipa,
        routes::health,
        routes::setting,
        routes::index::get_index_info,
        routes::index::get_documents,
        routes::index::create_index,
        routes::index::search_code,
        routes::index::search_files,
        routes::index::semantic_search,
        routes::index::get_index_status,
        routes::index::delete_index,
        routes::index::rebuild_index,
        routes::index::get_index_config,
        routes::index::validate_config,
        routes::index::analyze_code,
        routes::index::get_suggestions,
        routes::index::batch_create_index,
        routes::index::get_batch_status,
        tabby_webserver::routes::get_user_token,
        tabby_webserver::routes::execute_graphql_http,
        tabby_webserver::routes::register_user,
    ),
    components(schemas(
        api::event::LogEventRequest,
        completion::CompletionRequest,
        completion::CompletionResponse,
        completion::Segments,
        completion::Declaration,
        completion::Choice,
        completion::Snippet,
        completion::DebugOptions,
        completion::DebugData,
        completion::EditHistory,
        health::HealthState,
        health::Version,
        api::server_setting::ServerSetting,
        routes::index::IndexInfo,
        routes::index::DocumentInfo,
        routes::index::CreateIndexRequest,
        routes::index::IndexingProgress,
        routes::index::SearchRequest,
        routes::index::SearchResult,
        routes::index::SearchResponse,
        routes::index::FileSearchQuery,
        routes::index::IndexStatus,
        routes::index::IndexConfig,
        routes::index::ConfigValidationResponse,
        routes::index::AnalyzeRequest,
        routes::index::AnalyzeResponse,
        routes::index::IndexSuggestion,
        routes::index::BatchCreateRequest,
        routes::index::BatchStatus,
        routes::index::CreateIndexResponse,
        tabby_webserver::routes::GetTokenRequest,
        tabby_webserver::routes::GetTokenResponse,
        tabby_webserver::routes::GraphqlHttpRequest,
        tabby_webserver::routes::RegisterRequest,
        tabby_webserver::routes::RegisterResponse,
    )),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

#[derive(Args)]
pub struct ServeArgs {
    /// Model id for `/completions` API endpoint.
    #[clap(long)]
    model: Option<String>,

    /// Model id for `/chat/completions` API endpoints.
    #[clap(long)]
    chat_model: Option<String>,

    #[clap(long, default_value = "0.0.0.0")]
    host: IpAddr,

    #[clap(long, default_value_t = 8080)]
    port: u16,

    /// Device to run model inference.
    #[clap(long, default_value_t=Device::Cpu)]
    device: Device,

    /// Device to run chat model [default equals --device arg]
    #[clap(long, requires("chat_model"))]
    chat_device: Option<Device>,

    /// Parallelism for model serving - increasing this number will have a significant impact on the
    /// memory requirement e.g., GPU vRAM.
    #[clap(long, default_value_t = 1)]
    parallelism: u8,

    #[cfg(feature = "ee")]
    #[clap(hide = true, long, default_value_t = false)]
    no_webserver: bool,
}

pub async fn main(config: &Config, args: &ServeArgs) {
    let config = merge_args(config, args);

    load_model(&config).await;

    let tx = try_run_spinner();

    #[allow(unused_assignments)]
    let mut webserver = None;

    #[cfg(feature = "ee")]
    {
        webserver = Some(!args.no_webserver)
    }

    let embedding = embedding::create(&config.model.embedding).await;

    #[cfg(feature = "ee")]
    let ws = if !args.no_webserver {
        Some(
            tabby_webserver::public::Webserver::new(create_event_logger(), embedding.clone()).await,
        )
    } else {
        None
    };

    let mut logger: Arc<dyn EventLogger> = Arc::new(create_event_logger());

    #[cfg(feature = "ee")]
    if let Some(ws) = &ws {
        logger = ws.logger();
    }

    let index_reader_provider = Arc::new(IndexReaderProvider::default());
    let docsearch = Arc::new(services::structured_doc::create(
        embedding.clone(),
        index_reader_provider.clone(),
    ));

    let code = Arc::new(create_code_search(
        embedding.clone(),
        index_reader_provider.clone(),
    ));

    let model = &config.model;
    let (completion, completion_stream, chat) = create_completion_service_and_chat(
        &config.completion,
        code.clone(),
        logger.clone(),
        model.completion.clone(),
        model.chat.clone(),
    )
    .await;

    let chat_state = chat.as_ref().map(|c| {
        Arc::new(ChatState {
            chat_completion: c.clone(),
            logger: logger.clone(),
        })
    });
    let mut api = api_router(
        args,
        &config,
        logger.clone(),
        code.clone(),
        completion,
        chat_state,
        webserver,
    )
    .await;
    let mut doc = ApiDoc::openapi();
    #[cfg(feature = "ee")]
    doc.merge(EEApiDoc::openapi());
    let mut ui = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", doc))
        .fallback(|| async { axum::response::Redirect::temporary("/swagger-ui") });

    #[cfg(feature = "ee")]
    if let Some(ws) = &ws {
        let (new_api, new_ui) = ws
            .attach(
                &config,
                api,
                ui,
                code,
                chat,
                completion_stream,
                docsearch,
                |x| Box::new(services::structured_doc::create_serper(x)),
            )
            .await;
        api = new_api;
        ui = new_ui;
    };

    if let Some(tx) = tx {
        tx.send(())
            .unwrap_or_else(|_| warn!("Spinner channel is closed"));
    }
    start_heartbeat(args, &config, webserver);
    run_app(api, Some(ui), args.host, args.port).await
}

async fn load_model(config: &Config) {
    if let Some(ModelConfig::Local(ref model)) = config.model.completion {
        download_model_if_needed(&model.model_id, ModelKind::Completion).await;
    }

    if let Some(ModelConfig::Local(ref model)) = config.model.chat {
        download_model_if_needed(&model.model_id, ModelKind::Chat).await;
    }

    if let ModelConfig::Local(ref model) = config.model.embedding {
        download_model_if_needed(&model.model_id, ModelKind::Embedding).await;
    }
}

async fn api_router(
    args: &ServeArgs,
    config: &Config,
    logger: Arc<dyn EventLogger>,
    _code: Arc<dyn CodeSearch>,
    completion_state: Option<CompletionService>,
    chat_state: Option<Arc<ChatState>>,
    webserver: Option<bool>,
) -> Router {
    let mut router = Router::new();

    // 添加基础路由
    router = router
        .route("/health", routing::get(routes::health))
        .route("/v1/events", routing::post(routes::log_event).with_state(logger.clone()))
        .route("/v1/setting", routing::get(routes::setting))
        .route("/v1/index/info", routing::get(routes::index::get_index_info))
        .route("/v1/index/documents", routing::get(routes::index::get_documents))
        .route("/v1/index/create", routing::post(routes::index::create_index))
        .route("/v1/index/search", routing::post(routes::index::search_code))
        .route("/v1/index/files", routing::post(routes::index::search_files))
        .route("/v1/index/semantic", routing::post(routes::index::semantic_search))
        .route("/v1/index/status", routing::get(routes::index::get_index_status))
        .route("/v1/index/delete", routing::delete(routes::index::delete_index))
        .route("/v1/index/rebuild", routing::post(routes::index::rebuild_index))
        .route("/v1/index/config", routing::get(routes::index::get_index_config))
        .route("/v1/index/validate", routing::post(routes::index::validate_config))
        .route("/v1/index/analyze", routing::post(routes::index::analyze_code))
        .route("/v1/index/suggestions", routing::get(routes::index::get_suggestions))
        .route("/v1/index/batch", routing::post(routes::index::batch_create_index))
        .route("/v1/index/batch/status", routing::get(routes::index::get_batch_status));

    // 添加测试代理路由
    if let Some(completion_service) = &completion_state {
        let test_agent = TestAgentService::new(Arc::new(completion_service.clone()));
        router = router.merge(test_agent::router(test_agent));
    }

    // 添加健康检查路由
    let health_state = Arc::new(health::HealthState::new(
        config.server.model.clone(),
        config.server.chat_model.clone(),
    ));
    router = router
        .route("/v1/health", routing::get(routes::health).with_state(health_state.clone()))
        .route("/v1/health", routing::post(routes::health).with_state(health_state));

    // 添加模型路由
    let model_info = Arc::new(routes::models::ModelInfo::from_config(config));
    router = router.route("/v1beta/models", routing::get(routes::models).with_state(model_info));

    // 添加补全路由
    if let Some(completion_service) = completion_state {
        router = router.route(
            "/v1/completions",
            routing::post(routes::completions).with_state(Arc::new(completion_service)),
        );
    }

    // 添加聊天路由
    if let Some(chat_state) = chat_state {
        router = router
            .route(
                "/v1/chat/completions",
                routing::post(routes::chat_completions).with_state(chat_state.clone()),
            )
            .route(
                "/v1beta/chat/completions",
                routing::post(routes::chat_completions).with_state(chat_state),
            );
    } else {
        router = router
            .route("/v1/chat/completions", routing::post(StatusCode::NOT_IMPLEMENTED))
            .route("/v1beta/chat/completions", routing::post(StatusCode::NOT_IMPLEMENTED));
    }

    // 添加服务器设置路由
    let server_setting_router = routes::server_setting::router();
    #[cfg(feature = "ee")]
    if args.no_webserver {
        router = router.merge(server_setting_router);
    }

    #[cfg(not(feature = "ee"))]
    router = router.merge(server_setting_router);

    router
}

fn start_heartbeat(args: &ServeArgs, config: &Config, webserver: Option<bool>) {
    let state = Arc::new(health::HealthState::new(
        &config.model,
        &args.device,
        args.chat_model
            .as_deref()
            .map(|_| args.chat_device.as_ref().unwrap_or(&args.device)),
        webserver,
    ));
    tokio::spawn(async move {
        loop {
            usage::capture("ServeHealth", &state).await;
            sleep(Duration::from_secs(3000)).await;
        }
    });
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("token")
                        .build(),
                ),
            )
        }
    }
}

fn merge_args(config: &Config, args: &ServeArgs) -> Config {
    let mut config = (*config).clone();
    if let Some(model) = &args.model {
        if config.model.completion.is_some() {
            warn!("Overriding completion model from config.toml. The overriding behavior might surprise you. Consider setting the model in config.toml directly.");
        }
        config.model.completion = Some(to_local_config(model, args.parallelism, &args.device));
    };

    if let Some(chat_model) = &args.chat_model {
        if config.model.chat.is_some() {
            warn!("Overriding chat model from config.toml. The overriding behavior might surprise you. Consider setting the model in config.toml directly.");
        }
        config.model.chat = Some(to_local_config(
            chat_model,
            args.parallelism,
            args.chat_device.as_ref().unwrap_or(&args.device),
        ));
    }

    config
}

fn try_run_spinner() -> Option<Sender<()>> {
    if cfg!(feature = "prod") {
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::task::spawn(async move {
            let mut sp = Spinner::with_timer_and_stream(
                Spinners::Dots,
                "Starting...".into(),
                Stream::Stdout,
            );
            let _ = rx.await;
            sp.stop_with_message("".into());
        });
        Some(tx)
    } else {
        debug!("Starting server, this might take a few minutes...");
        None
    }
}
