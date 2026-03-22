use clap::Parser;
use semantic_core::ModelConfiguration;
use std::path::PathBuf;

mod server_arguments;

fn main() {
    let server_arguments = server_arguments::ServerArguments::parse();
    let models_dir = read_models(server_arguments.models_dir()).unwrap();
    println!(
        "Loaded {} models from {}",
        models_dir.len(),
        server_arguments.models_dir().display()
    )
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
    IoError(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
