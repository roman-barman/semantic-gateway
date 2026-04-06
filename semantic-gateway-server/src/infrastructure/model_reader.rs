use semantic_core::ModelConfiguration;
use std::path::PathBuf;

pub(crate) fn read_models(path: &PathBuf) -> Result<Vec<ModelConfiguration>, ReadModelsError> {
    let mut models = Vec::new();
    for dir_entry in std::fs::read_dir(path).map_err(ReadModelsError::ReadDir)? {
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
pub(crate) enum ReadModelsError {
    #[error("Read model directory error: {0}")]
    ReadDir(std::io::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    ParseYaml(#[from] serde_yaml::Error),
}
