use crate::semantic_configuration::Aggregate;
use crate::semantic_layer::query::Query;
use crate::semantic_layer::query_result::{QueryResult, QueryResultError};
use crate::semantic_layer::semantic_layer_info::SemanticLayerInfo;
use datafusion::arrow::array::{Float64Array, Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::datasource::MemTable;
use datafusion::error::DataFusionError;
use datafusion::prelude::{DataFrame, Expr, SessionContext, col};
use std::sync::Arc;

pub struct SemanticLayerContext<'a> {
    semantic_layer_info: &'a SemanticLayerInfo,
    context: SessionContext,
}

impl<'a> SemanticLayerContext<'a> {
    pub fn new(semantic_layer_info: &'a SemanticLayerInfo) -> Self {
        SemanticLayerContext {
            semantic_layer_info,
            context: SessionContext::new(),
        }
    }

    pub async fn execute_query(
        self,
        query: &Query<'_>,
    ) -> Result<QueryResult, ExecutionQueryError> {
        self.context
            .register_table("orders", create_orders_table())
            .map_err(ExecutionQueryError::RegisterTable)?;
        let df = self.build_dataframe(query).await?;
        let result = df
            .collect()
            .await
            .map_err(ExecutionQueryError::QueryExecution)?;

        Ok(QueryResult::try_from(result)?)
    }

    async fn build_dataframe(&self, query: &Query<'a>) -> Result<DataFrame, ExecutionQueryError> {
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
            .table(table)
            .await
            .map_err(ExecutionQueryError::DataFrameCreation)?;
        let group_by: Vec<Expr> = query
            .dimensions()
            .iter()
            .filter_map(|dimension| {
                self.semantic_layer_info
                    .get_dimension_column(dimension.model(), dimension.name())
                    .map(|field| col(field.as_ref()))
            })
            .collect();
        let aggregate: Vec<Expr> = query
            .metrics()
            .iter()
            .filter_map(|metric| {
                self.semantic_layer_info
                    .get_metric_info(metric.model(), metric.name())
                    .map(|(aggregate, field)| {
                        aggregate_expr(aggregate, field.as_ref(), metric.name())
                    })
            })
            .collect();

        df = df
            .aggregate(group_by, aggregate)
            .map_err(ExecutionQueryError::AggregationCreation)?;

        Ok(df)
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

fn create_orders_table() -> Arc<MemTable> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("order_id", DataType::Int64, false),
        Field::new("country", DataType::Utf8, false),
        Field::new("amount", DataType::Float64, false),
        Field::new("status", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5, 6])),
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
    .expect("Failed to create RecordBatch");
    Arc::new(MemTable::try_new(schema, vec![vec![batch]]).expect("Failed to create MemTable"))
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionQueryError {
    #[error("Failed to register table: {0}")]
    RegisterTable(DataFusionError),
    #[error("Empty query")]
    EmptyQuery,
    #[error("Failed to create DataFrame: {0}")]
    DataFrameCreation(DataFusionError),
    #[error("Failed to create aggregation: {0}")]
    AggregationCreation(DataFusionError),
    #[error("Failed to execute query: {0}")]
    QueryExecution(DataFusionError),
    #[error("Invalid model: {0}")]
    InvalidModel(String),
    #[error("Failed to parse query result: {0}")]
    QueryResult(#[from] QueryResultError),
}
