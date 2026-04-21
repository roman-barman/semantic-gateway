use semantic_core::ModelConfiguration;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) async fn read_models(
    path: &PathBuf,
) -> Result<HashMap<String, ModelConfiguration>, ReadModelsError> {
    let mut models = HashMap::new();
    let mut entries = tokio::fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            let path = entry.path();
            let extension = path.extension();
            if let Some(extension) = extension.and_then(|e| e.to_str())
                && (extension == "yaml" || extension == "yml")
            {
                let name = path
                    .file_stem()
                    .ok_or(ReadModelsError::FileName)?
                    .to_str()
                    .ok_or(ReadModelsError::FileName)?
                    .to_string();
                let file = std::fs::File::open(path)?;
                let model = serde_yaml::from_reader(file)?;
                models.insert(name, model);
            }
        }
    }

    Ok(models)
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ReadModelsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error("Can not read file name")]
    FileName,
}
