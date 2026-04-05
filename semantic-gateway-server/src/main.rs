use crate::infrastructure::initialize_tracing_subscribe;
use actix_web::{App, HttpServer};
use clap::Parser;
use semantic_core::ModelConfiguration;
use std::path::PathBuf;
use tracing::info;
use tracing_actix_web::TracingLogger;

mod configuration;
mod infrastructure;
mod server_arguments;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_arguments = server_arguments::ServerArguments::parse();
    let config = configuration::Configuration::read_configuration()?;
    initialize_tracing_subscribe(config.server().log_level())?;
    let models = read_models(server_arguments.models_dir())?;
    info!("Loaded {} models", models.len());

    HttpServer::new(|| App::new().wrap(TracingLogger::default()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}

fn read_models(path: &PathBuf) -> Result<Vec<ModelConfiguration>, BuildModelsError> {
    let mut models = Vec::new();
    for dir_entry in std::fs::read_dir(path)? {
        let dir_entry = dir_entry?;

        if dir_entry.file_type()?.is_file() {
            let path = dir_entry.path();
            let extension = path.extension();
            if let Some(extension) = extension.and_then(|e| e.to_str())
                && (extension == "yaml" || extension == "yml")
            {
                let file = std::fs::File::open(path)?;
                let model = serde_yaml::from_reader(file)?;
                models.push(model);
            }
        }
    }

    Ok(models)
}

#[derive(thiserror::Error, Debug)]
enum BuildModelsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
