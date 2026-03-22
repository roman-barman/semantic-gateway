#[derive(serde::Deserialize, Clone)]
pub(crate) struct Configuration {
    server: ServerConfiguration,
}

impl Configuration {
    pub(crate) fn read_configuration() -> Result<Self, ConfigurationError> {
        let env = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let base_path = std::env::current_dir()?;
        let config_dir = base_path.join("config");
        let env_file = format!("{}.yaml", env);
        let configuration = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base.yaml")))
            .add_source(config::File::from(config_dir.join(env_file)))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;
        let result = configuration.try_deserialize::<Configuration>()?;
        Ok(result)
    }

    pub(crate) fn server(&self) -> &ServerConfiguration {
        &self.server
    }
}

#[derive(serde::Deserialize, Clone)]
pub(crate) struct ServerConfiguration {
    log_level: String,
}

impl ServerConfiguration {
    pub(crate) fn log_level(&self) -> &str {
        &self.log_level
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ConfigurationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
}
