//! Data source abstraction for registering datasets into a DataFusion session.

mod parquet;

pub use parquet::{ParquetDataSource, ParquetDataSourceError};

use datafusion::error::DataFusionError;
use datafusion::prelude::SessionContext;
use std::path::PathBuf;

/// Errors that can occur while registering a data source into a DataFusion session.
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

/// Registers data into a DataFusion [`SessionContext`] for query execution.
///
/// Implementors scan a backing store and register each discovered dataset as a
/// named table in `ctx` so it can be referenced in SQL queries.
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    /// Scans the backing store and registers each dataset as a named table in `ctx`.
    async fn register(&self, ctx: &SessionContext) -> Result<(), DataSourceError>;
}
