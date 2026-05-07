use super::{DataSource, DataSourceError};
use datafusion::prelude::SessionContext;
use std::path::PathBuf;

/// Registers Parquet files from a directory as DataFusion tables.
///
/// Each `*.parquet` file in the directory is registered as a table named after
/// its file stem (e.g. `orders.parquet` → table `orders`). Subdirectories and
/// non-`.parquet` files are silently skipped.
#[derive(Debug)]
pub struct ParquetDataSource {
    data_dir: PathBuf,
}

impl ParquetDataSource {
    /// Creates a new `ParquetDataSource` rooted at `data_dir`.
    ///
    /// Returns [`ParquetDataSourceError::NotADirectory`] if `data_dir` does not
    /// exist or is not a directory.
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

/// Errors that can occur when constructing a [`ParquetDataSource`].
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParquetDataSourceError {
    #[error("data source path is not a directory: {0}")]
    NotADirectory(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::prelude::SessionContext;
    use std::path::Path;

    fn write_empty_parquet(path: &Path) {
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::parquet::arrow::ArrowWriter;
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
        let file = std::fs::File::create(path).unwrap();
        ArrowWriter::try_new(file, schema, None)
            .unwrap()
            .close()
            .unwrap();
    }

    fn table_names(ctx: &SessionContext) -> Vec<String> {
        ctx.catalog("datafusion")
            .unwrap()
            .schema("public")
            .unwrap()
            .table_names()
    }

    #[test]
    fn new_returns_error_for_nonexistent_path() {
        let path = PathBuf::from("/tmp/semantic_gateway_definitely_nonexistent_dir_xyzzy");
        assert_eq!(
            ParquetDataSource::new(path.clone()).unwrap_err(),
            ParquetDataSourceError::NotADirectory(path)
        );
    }

    #[test]
    fn new_returns_error_when_path_is_a_file() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        assert_eq!(
            ParquetDataSource::new(path.clone()).unwrap_err(),
            ParquetDataSourceError::NotADirectory(path)
        );
    }

    #[test]
    fn new_succeeds_for_valid_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(ParquetDataSource::new(dir.path().to_path_buf()).is_ok());
    }

    #[tokio::test]
    async fn register_on_empty_directory_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let ds = ParquetDataSource::new(dir.path().to_path_buf()).unwrap();
        let ctx = SessionContext::new();
        ds.register(&ctx).await.unwrap();
        assert!(table_names(&ctx).is_empty());
    }

    #[tokio::test]
    async fn register_skips_non_parquet_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("data.csv"), "id,name\n1,test").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "hello").unwrap();
        let ds = ParquetDataSource::new(dir.path().to_path_buf()).unwrap();
        let ctx = SessionContext::new();
        ds.register(&ctx).await.unwrap();
        assert!(table_names(&ctx).is_empty());
    }

    #[tokio::test]
    async fn register_skips_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        let ds = ParquetDataSource::new(dir.path().to_path_buf()).unwrap();
        let ctx = SessionContext::new();
        ds.register(&ctx).await.unwrap();
        assert!(table_names(&ctx).is_empty());
    }

    #[tokio::test]
    async fn register_creates_table_named_after_file_stem() {
        let dir = tempfile::tempdir().unwrap();
        write_empty_parquet(&dir.path().join("orders.parquet"));
        let ds = ParquetDataSource::new(dir.path().to_path_buf()).unwrap();
        let ctx = SessionContext::new();
        ds.register(&ctx).await.unwrap();
        assert_eq!(table_names(&ctx), vec!["orders"]);
    }

    #[tokio::test]
    async fn register_all_parquet_files_in_directory() {
        let dir = tempfile::tempdir().unwrap();
        write_empty_parquet(&dir.path().join("orders.parquet"));
        write_empty_parquet(&dir.path().join("products.parquet"));
        let ds = ParquetDataSource::new(dir.path().to_path_buf()).unwrap();
        let ctx = SessionContext::new();
        ds.register(&ctx).await.unwrap();
        let mut names = table_names(&ctx);
        names.sort();
        assert_eq!(names, vec!["orders", "products"]);
    }
}
