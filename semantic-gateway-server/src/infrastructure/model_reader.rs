use semantic_core::ModelConfiguration;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) fn read_models(
    path: &PathBuf,
) -> Result<HashMap<String, ModelConfiguration>, ReadModelsError> {
    let mut models = HashMap::new();
    for dir_entry in std::fs::read_dir(path).map_err(ReadModelsError::ReadDir)? {
        let dir_entry = dir_entry?;

        if dir_entry.file_type()?.is_file() {
            let path = dir_entry.path();
            let extension = path.extension();
            if let Some(extension) = extension.and_then(|e| e.to_str())
                && (extension == "yaml" || extension == "yml")
            {
                let name = dir_entry
                    .file_name()
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
    #[error("Read model directory error: {0}")]
    ReadDir(std::io::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error("Can not read file name")]
    FileName,
}
