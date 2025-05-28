use std::sync::Arc;

use axum::Router;
use tabby_common::{
    api::{
        code::CodeSearch,
        event::{ComposedLogger, EventLogger},
        structured_doc::DocSearch,
    },
    config::Config,
};
use tabby_db::DbConn;
use tabby_inference::{ChatCompletionStream, CompletionStream, Embedding};
use tabby_schema::job::JobService;
use tracing::debug;

use crate::{
    path::db_file,
    routes,
    service::{
        create_service_locator, embedding, event_logger::create_event_logger, ingestion,
        integration, job, model_configuration,
        new_auth_service, new_email_service, new_license_service,
        new_setting_service, repository, web_documents,
    },
};

pub struct Webserver {
    db: DbConn,
    logger: Arc<dyn EventLogger>,
    embedding: Arc<dyn Embedding>,
}

impl Webserver {
    pub async fn new(
        logger1: impl EventLogger + 'static,
        embedding: Arc<dyn Embedding>,
    ) -> Arc<Self> {
        let db = DbConn::new(db_file().as_path())
            .await
            .expect("Must be able to initialize db");
        db.finalize_stale_job_runs()
            .await
            .expect("Must be able to finalize stale job runs");

        let logger2 = create_event_logger(db.clone());
        let logger = Arc::new(ComposedLogger::new(logger1, logger2));

        Arc::new(Webserver {
            db: db.clone(),
            logger,
            embedding,
        })
    }

    pub fn logger(&self) -> Arc<dyn EventLogger + 'static> {
        self.logger.clone()
    }

    pub async fn attach(
        &self,
        config: &Config,
        api: Router,
        ui: Router,
        code_search_service: Arc<dyn CodeSearch>,
        chat_stream_from_args: Option<Arc<dyn ChatCompletionStream>>,
        completion_stream_from_args: Option<Arc<dyn CompletionStream>>,
        docsearch: Arc<dyn DocSearch>,
        serper_factory_fn: impl Fn(&str) -> Box<dyn DocSearch>,
    ) -> (Router, Router) {
        let serper: Option<Box<dyn DocSearch>> =
            if let Ok(api_key) = std::env::var("SERPER_API_KEY") {
                debug!("Serper API key found, enabling serper...");
                Some(serper_factory_fn(&api_key))
            } else {
                None
            };

        let db = self.db.clone();
        let logger = self.logger();
        let job_service: Arc<dyn JobService> = Arc::new(job::create(db.clone()).await);

        let integration_service = Arc::new(integration::create(db.clone(), job_service.clone()));
        let repository_service = repository::create(db.clone(), integration_service.clone(), job_service.clone());

        let web_documents_service = Arc::new(web_documents::create(db.clone(), job_service.clone()));
        let ingestion_service = Arc::new(ingestion::create(db.clone()));

        let context_service = Arc::new(crate::service::context::create(
            repository_service.clone(),
            ingestion_service.clone(),
            web_documents_service.clone(),
            serper.is_some(),
        ));

        let mail_service = Arc::new(
            new_email_service(db.clone())
                .await
                .expect("failed to initialize mail service"),
        );
        let license_service = Arc::new(
            new_license_service(db.clone())
                .await
                .expect("failed to initialize license service"),
        );
        let setting_service = Arc::new(new_setting_service(db.clone()));
        let auth_service = Arc::new(new_auth_service(
            db.clone(),
            mail_service.clone(),
            license_service.clone(),
            setting_service.clone(),
        ));

        let retrieval_service = Arc::new(crate::service::retrieval::create(
            code_search_service.clone(),
            docsearch.clone(),
            serper,
            repository_service.clone(),
            setting_service.clone(),
        ));

        let answer_service = chat_stream_from_args.as_ref().map(|chat_s| {
            Arc::new(crate::service::answer::create(
                logger.clone(),
                &config.answer,
                auth_service.clone(),
                chat_s.clone(),
                retrieval_service.clone(),
                context_service.clone(),
            ))
        });

        let embedding_service = embedding::create(&config.embedding, self.embedding.clone());

        let service_locator = create_service_locator(
            logger.clone(),
            auth_service,
            chat_stream_from_args,
            completion_stream_from_args,
            code_search_service.clone(),
            repository_service.clone(),
            integration_service.clone(),
            ingestion_service,
            job_service.clone(),
            answer_service.clone(),
            retrieval_service,
            context_service.clone(),
            web_documents_service.clone(),
            mail_service,
            license_service,
            setting_service,
            model_configuration::create(Arc::new(db.clone())),
            db.clone(),
            embedding_service,
        )
        .await;

        routes::create(service_locator, api, ui, answer_service)
    }
}
