mod api;

use crate::configuration::Configuration;
use actix_web::{App, HttpServer};
use tokio::task::{JoinError, JoinHandle};
use tracing_actix_web::TracingLogger;

pub(crate) struct WebServer {
    server_task: JoinHandle<std::io::Result<()>>,
}

impl WebServer {
    pub(crate) fn start(config: &Configuration) -> Result<Self, WebServerError> {
        let server_task = HttpServer::new(|| {
            App::new()
                .wrap(TracingLogger::default())
                .service(api::health)
                .service(api::execute_query)
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
