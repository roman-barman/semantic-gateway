mod api;

use crate::configuration::Configuration;
use actix_web::{App, HttpServer, web};
use semantic_core::data_source::DataSource;
use semantic_core::{ModelConfiguration, SemanticLayerInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::{JoinError, JoinHandle};
use tracing_actix_web::TracingLogger;

pub(crate) struct WebServer {
    server_task: JoinHandle<std::io::Result<()>>,
}

impl WebServer {
    pub(crate) fn start(
        config: &Configuration,
        layer: HashMap<String, ModelConfiguration>,
        data_source: Arc<dyn DataSource>,
    ) -> Result<Self, WebServerError> {
        let semantic_layer_info = web::Data::new(SemanticLayerInfo::new(layer));
        let data_source = web::Data::from(data_source);

        let server_task = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .service(api::health)
                .service(api::execute_query)
                .app_data(semantic_layer_info.clone())
                .app_data(data_source.clone())
        })
        .bind(config.server().address())
        .map_err(WebServerError::BindAddress)?
        .run();

        Ok(Self {
            server_task: tokio::spawn(server_task),
        })
    }

    pub(crate) async fn stop(self) -> Result<(), WebServerError> {
        self.server_task
            .await
            .map_err(WebServerError::Stop)?
            .map_err(WebServerError::Internal)
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum WebServerError {
    #[error("Failed to bind to address: {0}")]
    BindAddress(std::io::Error),
    #[error("Failed to stop server: {0}")]
    Stop(JoinError),
    #[error("Internal error: {0}")]
    Internal(std::io::Error),
}
