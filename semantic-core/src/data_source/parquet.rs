use super::{DataSource, DataSourceError};
use datafusion::prelude::SessionContext;
use std::path::PathBuf;

pub struct ParquetDataSource {
    data_dir: PathBuf,
}

impl ParquetDataSource {
    pub fn new(data_dir: PathBuf) -> Result<Self, ParquetDataSourceError> {
        if !data_dir.is_dir() {
            return Err(ParquetDataSourceError::NotADirectory(data_dir));
        }
        Ok(Self { data_dir })
    }
}

#[async_trait::async_trait]
impl DataSource for ParquetDataSource {
    async fn register(&self, ctx: &SessionContext) -> Result<(), DataSourceError> {
        let mut entries = tokio::fs::read_dir(&self.data_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("parquet") || !file_type.is_file()
            {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| DataSourceError::InvalidFileName(path.clone()))?
                .to_string();
            let path_str = path
                .to_str()
                .ok_or_else(|| DataSourceError::InvalidFileName(path.clone()))?;
            tracing::debug!(table = %name, path = %path.display(), "registering parquet table");
            ctx.register_parquet(&name, path_str, Default::default())
                .await
                .map_err(|source| DataSourceError::RegisterTable {
                    table: name,
                    source,
                })?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParquetDataSourceError {
    #[error("Data source path is not a directory: {0}")]
    NotADirectory(PathBuf),
}
