use datafusion::arrow::array::{Float64Array, Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::datasource::MemTable;
use datafusion::prelude::SessionContext;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    )?;

    let ctx = SessionContext::new();
    let table = MemTable::try_new(schema.clone(), vec![vec![batch]])?;
    ctx.register_table("orders", Arc::new(table))?;
    let df = ctx.table("orders").await?;
    df.write_parquet(
        "data/orders.parquet",
        datafusion::dataframe::DataFrameWriteOptions::default(),
        None,
    )
    .await?;

    println!("Generated ../data/orders.parquet");
    Ok(())
}
