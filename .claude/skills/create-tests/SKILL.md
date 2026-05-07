---
name: create-tests
description: Write unit tests for this Rust project. Use when the user asks to add, write, or create tests for any module or function. Enforces Arrange/Act/Assert structure and runs cargo nextest to verify.
user-invocable: true
---

# Create Tests

Write inline unit tests for the target code, then verify them with `cargo nextest run`.

## Step 1: Read the target file

Read the file the user wants to test. Identify what is worth covering: public functions, error branches, edge cases, and boundary conditions.

## Step 2: Place tests correctly

Append a `#[cfg(test)]` module at the **bottom of the same source file** — never in a separate file. `use super::*;` must always be the first import inside the module.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // additional imports...
}
```

## Step 3: Structure every test with Arrange / Act / Assert

Each test body **must** contain exactly these three sections, marked with inline comments:

```rust
#[test]
fn descriptive_scenario_name() {
    // Arrange
    // build inputs, create fixtures, initialize helpers

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected);
}
```

For async tests use `#[tokio::test]`:

```rust
#[tokio::test]
async fn query_groups_revenue_by_country() {
    // Arrange
    let info = make_semantic_layer_info();
    let data_source = MemDataSource { table_name: "orders".to_string() };
    let ctx = SemanticLayerContext::new(Arc::new(info), Arc::new(data_source));
    let query = Query::new(
        vec![Metric::new("revenue", "orders")],
        vec![Dimension::new("country", "orders")],
        vec![],
    );

    // Act
    let result = ctx.execute_query(&query).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().row_count(), 2);
}
```

## Step 4: Choose the right fixture pattern

| What you're testing | Fixture to use |
|---|---|
| Config / model deserialization | YAML raw string + `serde_yaml::from_str(yaml).unwrap()` in a helper fn |
| Query execution (DataFusion) | Local `MemDataSource` struct implementing `DataSource` trait with `MemTable` |
| File system (parquet scan, path logic) | `tempfile::tempdir()` and write files into it |

### MemDataSource template (copy per test module — never share across modules)

```rust
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
                Arc::new(Int64Array::from(vec![1i64, 2, 3])),
                Arc::new(StringArray::from(vec!["GE", "RU", "US"])),
                Arc::new(Float64Array::from(vec![150.0, 350.0, 500.0])),
                Arc::new(StringArray::from(vec!["completed", "completed", "completed"])),
            ],
        )
        .map_err(|e| DataSourceError::RegisterTable {
            table: self.table_name.clone(),
            source: e.into(),
        })?;
        let table = Arc::new(
            MemTable::try_new(schema, vec![vec![batch]]).map_err(|source| {
                DataSourceError::RegisterTable {
                    table: self.table_name.clone(),
                    source,
                }
            })?,
        );
        ctx.register_table(&self.table_name, table).map_err(|source| {
            DataSourceError::RegisterTable {
                table: self.table_name.clone(),
                source,
            }
        })?;
        Ok(())
    }
}
```

## Step 5: Naming rules

- snake_case, no `test_` prefix
- Name the scenario, not the function: `revenue_is_summed_across_rows`, `unknown_metric_returns_error`

## Step 6: `unwrap()` placement

- `unwrap()` is allowed only inside fixture helper functions (e.g., `make_semantic_layer_info()`)
- In the test body itself, keep Arrange/Act clean: `.await` the result in Act, then assert on it in Assert

## Step 7: Run and fix

After writing the tests, run:

```bash
cargo nextest run
```

For a single crate: `cargo nextest run -p semantic-core`

Fix all compilation errors and test failures before reporting done. Do not report the task complete until `cargo nextest run` exits with code 0.
