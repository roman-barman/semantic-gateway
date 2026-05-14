mod factory;

use crate::data_source::{DataSource, DataSourceError};
use crate::semantic_layer::filter::{Filter, FilterOperation, FilterValue};
use crate::semantic_layer::layer_info::{Aggregate, SemanticLayerInfo};
use crate::semantic_layer::query::Query;
use crate::semantic_layer::query_result::{QueryResult, QueryResultError};
use datafusion::error::DataFusionError;
use datafusion::logical_expr::lit;
use datafusion::logical_expr::utils::conjunction;
use datafusion::prelude::{DataFrame, Expr, SessionContext, col};
pub use factory::SemanticLayerContextFactory;
use std::sync::Arc;

pub struct SemanticLayerContext {
    semantic_layer_info: Arc<SemanticLayerInfo>,
    context: SessionContext,
    data_source: Arc<dyn DataSource>,
}

impl SemanticLayerContext {
    pub fn new(
        semantic_layer_info: Arc<SemanticLayerInfo>,
        data_source: Arc<dyn DataSource>,
    ) -> Self {
        SemanticLayerContext {
            semantic_layer_info,
            context: SessionContext::new(),
            data_source,
        }
    }

    pub fn new_with_context(
        session_context: SessionContext,
        semantic_layer_info: Arc<SemanticLayerInfo>,
        data_source: Arc<dyn DataSource>,
    ) -> Self {
        SemanticLayerContext {
            semantic_layer_info,
            context: session_context,
            data_source,
        }
    }

    pub async fn execute_query(
        self,
        query: &Query<'_>,
    ) -> Result<QueryResult, ExecutionQueryError> {
        self.data_source.register(&self.context).await?;
        let df = self.build_dataframe(query).await?;
        let result = df
            .collect()
            .await
            .map_err(ExecutionQueryError::QueryExecution)?;

        Ok(QueryResult::try_from(result)?)
    }

    async fn build_dataframe(&self, query: &Query<'_>) -> Result<DataFrame, ExecutionQueryError> {
        let model = query
            .models()
            .iter()
            .next()
            .ok_or(ExecutionQueryError::EmptyQuery)?
            .to_string();
        let table = self
            .semantic_layer_info
            .table(model.as_ref())
            .ok_or(ExecutionQueryError::InvalidModel(model.to_string()))?;
        let mut df = self
            .context
            .table(table.as_ref())
            .await
            .map_err(ExecutionQueryError::DataFrameCreation)?;
        let group_by: Vec<Expr> = query
            .dimensions()
            .iter()
            .filter_map(|dimension| {
                self.semantic_layer_info
                    .dimension_config(dimension)
                    .map(|config| col(config.field().as_ref()))
            })
            .collect();
        let aggregate: Vec<Expr> = query
            .metrics()
            .iter()
            .filter_map(|metric| {
                self.semantic_layer_info
                    .metric_config(metric)
                    .map(|config| {
                        aggregate_expr(config.aggregate(), config.field().as_ref(), metric.name())
                    })
            })
            .collect();

        let filters: Vec<Expr> = query
            .filters()
            .iter()
            .map(|filter| self.filter_expr(filter))
            .collect::<Result<_, _>>()?;

        if let Some(predicate) = conjunction(filters) {
            df = df
                .filter(predicate)
                .map_err(ExecutionQueryError::FilterCreation)?;
        }

        df = df
            .aggregate(group_by, aggregate)
            .map_err(ExecutionQueryError::AggregationCreation)?;

        Ok(df)
    }

    fn filter_expr(&self, filter: &Filter<'_>) -> Result<Expr, ExecutionQueryError> {
        let field = self
            .semantic_layer_info
            .column(filter.model(), filter.field())
            .ok_or(ExecutionQueryError::InvalidFiled {
                model: filter.model().to_string(),
                name: filter.field().to_string(),
            })?;
        let field = col(field);
        let value = match filter.value() {
            FilterValue::String(value) => lit(value.to_string()),
            FilterValue::Int(value) => lit(*value),
            FilterValue::Float(value) => lit(*value),
        };

        let exp = match filter.operation() {
            FilterOperation::Eq => field.eq(value),
            FilterOperation::Ne => field.not_eq(value),
            FilterOperation::Gt => field.gt(value),
            FilterOperation::Lt => field.lt(value),
            FilterOperation::Gte => field.gt_eq(value),
            FilterOperation::Lte => field.lt_eq(value),
        };

        Ok(exp)
    }
}

fn aggregate_expr(aggregation: &Aggregate, field: &str, alias: &str) -> Expr {
    let field = col(field);
    let expr = match aggregation {
        Aggregate::Sum => datafusion::functions_aggregate::sum::sum(field),
        Aggregate::Count => datafusion::functions_aggregate::count::count(field),
    };

    expr.alias(alias)
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionQueryError {
    #[error("data source error: {0}")]
    DataSource(#[from] DataSourceError),
    #[error("empty query")]
    EmptyQuery,
    #[error("failed to create DataFrame: {0}")]
    DataFrameCreation(DataFusionError),
    #[error("failed to create aggregation: {0}")]
    AggregationCreation(DataFusionError),
    #[error("failed to create filter: {0}")]
    FilterCreation(DataFusionError),
    #[error("failed to execute query: {0}")]
    QueryExecution(DataFusionError),
    #[error("invalid model: {0}")]
    InvalidModel(String),
    #[error("failed to parse query result: {0}")]
    QueryResult(#[from] QueryResultError),
    #[error("invalid field: model '{model}', field '{name}'")]
    InvalidFiled { model: String, name: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_source::DataSourceError;
    use crate::semantic_layer::layer_info::{ModelConfiguration, SemanticLayerInfo};
    use crate::semantic_layer::query::Query;
    use crate::{Dimension, Metric};
    use datafusion::arrow::array::{Float64Array, Int64Array, RecordBatch, StringArray};
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use datafusion::datasource::MemTable;
    use datafusion::prelude::SessionContext;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MemDataSource {
        table_name: String,
    }

    #[async_trait::async_trait]
    impl DataSource for MemDataSource {
        async fn register(&self, ctx: &SessionContext) -> Result<(), DataSourceError> {
            let schema = Arc::new(Schema::new(vec![
                Field::new("order_id", DataType::Int64, false),
                Field::new("country", DataType::Utf8, false),
                Field::new("amount", DataType::Float64, false),
                Field::new("status", DataType::Utf8, false),
            ]));
            let batch = RecordBatch::try_new(
                schema.clone(),
                vec![
                    Arc::new(Int64Array::from(vec![1i64, 2, 3, 4, 5, 6])),
                    Arc::new(StringArray::from(vec!["GE", "GE", "RU", "RU", "US", "US"])),
                    Arc::new(Float64Array::from(vec![
                        150.0, 200.0, 350.0, 100.0, 500.0, 750.0,
                    ])),
                    Arc::new(StringArray::from(vec![
                        "completed",
                        "completed",
                        "completed",
                        "cancelled",
                        "completed",
                        "completed",
                    ])),
                ],
            )
            .map_err(|e| DataSourceError::RegisterTable {
                table: self.table_name.clone(),
                source: e.into(),
            })?;
            let table = Arc::new(MemTable::try_new(schema, vec![vec![batch]]).map_err(
                |source| DataSourceError::RegisterTable {
                    table: self.table_name.clone(),
                    source,
                },
            )?);
            ctx.register_table(&self.table_name, table)
                .map_err(|source| DataSourceError::RegisterTable {
                    table: self.table_name.clone(),
                    source,
                })?;
            Ok(())
        }
    }

    fn make_semantic_layer_info() -> SemanticLayerInfo {
        let yaml = r#"
table: orders
metrics:
  revenue:
    title: Revenue
    aggregate: sum
    field: amount
  orders_count:
    title: Orders Count
    aggregate: count
    field: order_id
dimensions:
  country:
    title: Country
    field: country
"#;
        let model: ModelConfiguration = serde_yaml::from_str(yaml).unwrap();
        SemanticLayerInfo::new(HashMap::from([("orders".to_string(), model)]))
    }

    #[tokio::test]
    async fn execute_query_groups_by_dimension() {
        let info = make_semantic_layer_info();
        let data_source = MemDataSource {
            table_name: "orders".to_string(),
        };
        let context = SemanticLayerContext::new(Arc::new(info), Arc::new(data_source));

        let metrics = vec![Metric::new("revenue", "orders")];
        let dimensions = vec![Dimension::new("country", "orders")];
        let filters = vec![Filter::new(
            "country",
            "orders",
            FilterOperation::Ne,
            FilterValue::String("US"),
        )];
        let query = Query::try_new(metrics, dimensions, filters).unwrap();

        let result = context.execute_query(&query).await.unwrap();
        assert_eq!(result.row_count(), 2);
    }
}
