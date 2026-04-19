use std::path::PathBuf;

/// Semantic gateway server
#[derive(clap::Parser, Debug)]
#[clap(about, version, long_about = None)]
pub struct ServerArguments {
    /// The directory containing the models
    #[clap(short, long, default_value = "models", value_parser = parse_models_dir)]
    models_dir: PathBuf,

    /// The directory containing parquet data files
    #[clap(long, default_value = "data", value_parser = parse_data_dir)]
    data_dir: PathBuf,
}

impl ServerArguments {
    pub fn models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}

fn parse_models_dir(path: &str) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("models directory path is empty".to_string());
    }
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!(
            "models directory '{}' does not exist",
            path.display()
        ));
    }
    if !path.is_dir() {
        return Err(format!(
            "models path '{}' is not a directory",
            path.display()
        ));
    }
    Ok(path)
}

fn parse_data_dir(path: &str) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("data directory path is empty".to_string());
    }
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!(
            "data directory '{}' does not exist",
            path.display()
        ));
    }
    if !path.is_dir() {
        return Err(format!("data path '{}' is not a directory", path.display()));
    }
    Ok(path)
}
