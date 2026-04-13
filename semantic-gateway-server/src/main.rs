use crate::infrastructure::{initialize_tracing_subscribe, read_models};
use clap::Parser;
use tracing::info;

mod configuration;
mod infrastructure;
mod server_arguments;
mod web_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_arguments = server_arguments::ServerArguments::parse();
    let config = configuration::Configuration::read_configuration()?;
    initialize_tracing_subscribe(config.server().log_level())?;
    let models = read_models(server_arguments.models_dir())?;
    info!("Loaded {} models", models.len());

    let server = web_server::WebServer::start(&config, models)?;
    tokio::signal::ctrl_c().await?;
    server.stop().await?;

    Ok(())
}
