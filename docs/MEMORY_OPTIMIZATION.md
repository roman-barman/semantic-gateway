# Memory Optimization

## Problem

The current `QueryResult` stores data as `HashMap<String, Vec<ColumnValue>>` where
`ColumnValue::String(String)` is an owned heap string. For every string cell during
serialization, `arr.value(i).into()` performs `String::from(&str)` — one heap allocation
per cell (`N_rows × N_string_columns` allocations per request).

Two additional minor allocation hotspots:
- `split_reference` in `execute_query.rs` collects a `Vec<&str>` for every metric/dimension reference
- `HashMap::new()` in `QueryResult::try_from` starts without capacity, causing reallocations

---

## Solution

### 1. Zero-copy string serialization via Arrow arrays

`StringArray::value(i)` returns `&str` borrowed from Arrow's internal buffer. Storing
`ArrayRef` (`Arc<dyn Array>`) instead of `Vec<ColumnValue>` allows serializing strings
directly from the Arrow buffer with no intermediate `String` allocation.

`RecordBatch::into_parts()` (arrow-array 57.3.0) consumes the batch and returns
`(SchemaRef, Vec<ArrayRef>, usize)` — a free O(1) operation that moves Arc pointers.

**New `QueryResult` shape:**

```rust
pub struct QueryResult {
    schema: Vec<ColumnMeta>,
    columns: Vec<(String, ArrayRef)>,  // order matches schema
    row_count: usize,
}
```

**`TryFrom` using `into_parts()`:**

```rust
let batch = concat_batches(&schema, &value).map_err(|_| QueryResultError::InvalidSchema)?;
let (schema, arrays, row_count) = batch.into_parts();

let schema_result = schema.fields().iter()
    .map(|f| ColumnMeta::new(f.name().clone(), arrow_type_to_value_type(f.data_type())))
    .collect();

let columns = schema.fields().iter()
    .zip(arrays)
    .map(|(f, arr)| (f.name().clone(), arr))
    .collect();
```

**Manual `Serialize` — strings written as `&str`, no `String` allocation:**

```rust
struct SerializableColumn<'a>(&'a dyn Array);

impl Serialize for SerializableColumn<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let arr = self.0;
        let mut seq = serializer.serialize_seq(Some(arr.len()))?;
        match arr.data_type() {
            DataType::Utf8 => {
                let arr = arr.as_any().downcast_ref::<StringArray>()
                    .ok_or_else(|| serde::ser::Error::custom("invalid array"))?;
                for i in 0..arr.len() {
                    if arr.is_null(i) { seq.serialize_element(&Option::<&str>::None)?; }
                    else              { seq.serialize_element(arr.value(i))?; } // &str — no allocation
                }
            }
            // Int*, UInt*, Float* — same pattern, Copy primitives widened to i64/u64/f64
        }
        seq.end()
    }
}

struct SerializableColumns<'a>(&'a [(String, ArrayRef)]);

impl Serialize for SerializableColumns<'_> {
    // serializes as { "col_name": [v1, v2, ...], ... }
}

impl Serialize for QueryResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("schema", &self.schema)?;
        map.serialize_entry("columns", &SerializableColumns(&self.columns))?;
        map.serialize_entry("row_count", &self.row_count)?;
        map.end()
    }
}
```

`ColumnValue` and its file (`column_value.rs`) can be deleted entirely.

---

### 2. `split_reference` → `split_once`

**File:** `semantic-gateway-server/src/web_server/api/execute_query.rs`

```rust
// before: allocates Vec<&str> for every "model.field" string
fn split_reference(value: &str) -> Vec<&str> {
    value.split('.').collect()
}

// after: returns Option<(&str, &str)>, no allocation
fn split_reference(value: &str) -> Option<(&str, &str)> {
    value.split_once('.')
}
```

Updated call site in `map_to_query`:

```rust
request.metrics.iter().map(|s| {
    split_reference(s)
        .ok_or_else(|| QueryError::InvalidMetric(s.clone()))
        .map(|(model, name)| Metric::new(name, model))
})
```

---

## Files Affected

| File | Change |
|------|--------|
| `semantic-core/src/semantic_layer/query_result.rs` | Major refactor: `into_parts()`, manual `Serialize` |
| `semantic-core/src/semantic_layer/query_result/column_value.rs` | Delete |
| `semantic-core/src/lib.rs` | Remove `pub use ColumnValue` if exported |
| `semantic-gateway-server/src/web_server/api/execute_query.rs` | `split_reference` → `split_once` |

---

## Outcome

| Optimization | Allocations eliminated |
|---|---|
| Arrow direct serialization | `N_rows × N_string_columns` per request |
| `into_parts()` | 0 (was already O(1) Arc clone; now O(1) Arc move) |
| `split_once` | `N_metrics + N_dimensions` Vec allocs per request |

JSON response format is unchanged.
