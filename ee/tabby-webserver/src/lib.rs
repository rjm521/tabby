//! Defines behavior for the tabby webserver which allows users to interact with enterprise features.
mod axum;
mod hub;
mod jwt;
mod ldap;
mod oauth;
mod path;
mod rate_limit;
pub mod routes;
mod service;
mod webserver;

#[cfg(test)]
pub use service::*;
use tabby_common::api;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::ingestion::ingestion,
        routes::ingestion::delete_ingestion_source,
        routes::ingestion::delete_ingestion,
        routes::ee_completions::ee_completions,
        routes::ee_chat::ee_chat_completions,
        routes::model_configuration::get_user_model_preference,
        routes::model_configuration::update_user_model_preference,
        routes::model_configuration::list_available_models,
        routes::model_configuration::get_available_model,
        routes::model_configuration::create_available_model,
        routes::model_configuration::update_available_model,
        routes::model_configuration::delete_available_model,
    ),
    components(schemas(
        api::ingestion::IngestionRequest,
        api::ingestion::IngestionResponse,
        routes::ee_completions::CompletionRequest,
        routes::ee_completions::CompletionResponse,
        routes::ee_chat::ChatRequest,
        routes::ee_chat::ChatMessage,
        routes::ee_chat::ChatResponse,
        routes::model_configuration::UserModelPreferenceResponse,
        routes::model_configuration::UpdateUserModelPreferenceRequest,
        routes::model_configuration::AvailableModelResponse,
        routes::model_configuration::CreateAvailableModelRequest,
        routes::model_configuration::UpdateAvailableModelRequest,
        routes::model_configuration::ListModelsQuery,
    )),
    // modifiers(&SecurityAddon),
)]
pub struct EEApiDoc;

pub mod public {

    pub use super::{
        hub::{create_worker_client, WorkerClient},
        webserver::Webserver,
    };
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return std::result::Result::Err(anyhow::anyhow!($msg).into())
    };
    ($err:expr $(,)?) => {
        return std::result::Result::Err(anyhow::anyhow!($err).into())
    };
    ($fmt:expr, $($arg:tt)*) => {
        return std::result::Result::Err(anyhow::anyhow!($fmt, $($arg)*).into())
    };
}
