mod parquet;

pub use parquet::{ParquetDataSource, ParquetDataSourceError};

use datafusion::error::DataFusionError;
use datafusion::prelude::SessionContext;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("failed to register table '{table}': {source}")]
    RegisterTable {
        table: String,
        #[source]
        source: DataFusionError,
    },
    #[error("non-UTF-8 path: {0}")]
    InvalidFileName(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    async fn register(&self, ctx: &SessionContext) -> Result<(), DataSourceError>;
}
