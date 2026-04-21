# Architecture

## Overview

`semantic-gateway` is a proof-of-concept semantic layer. The codebase is clean and compiles without warnings, but several gaps block production use. This document describes the current architecture and the planned improvements, ordered by priority.

---

## Current Architecture

### Crate Structure

```
semantic-gateway/
├── semantic-core/            # Query engine and semantic model
│   ├── data_source/          # DataSource trait + ParquetDataSource impl
│   └── semantic_layer/       # Query execution (DataFusion), metadata, result types
│       └── semantic_layer_info/  # YAML model deserialization (ModelConfiguration, etc.)
└── semantic-gateway-server/  # HTTP server, config, model loading
    ├── infrastructure/       # Model reader (YAML), tracing setup
    └── web_server/api/       # /health, /query/execute handlers
```

### Server Startup

1. CLI args parsed (`--models-dir`, `--data-dir`)
2. Configuration loaded (`config/base.yaml` → env override → `APP_*` env vars)
3. Tracing subscriber initialized
4. YAML models loaded via `read_models()` → `HashMap<String, ModelConfiguration>`
5. `ParquetDataSource::new(data_dir)` created, wrapped in `Arc<dyn DataSource>`
6. `SemanticLayerInfo` and `DataSource` stored as Actix `web::Data<>` (shared across requests)
7. `WebServer::start()` binds and runs

### Request Flow

```
POST /query/execute { metrics, dimensions }
    ↓
[execute_query handler]       — splits "model.field" refs, builds Query
    ↓
[SemanticLayerContext::new]   — receives SemanticLayerInfo + &dyn DataSource
    ↓
[data_source.register()]      — registers Parquet files as DataFusion tables
    ↓
[build_dataframe()]           — builds GROUP BY + aggregate logical plan
    ↓
[df.collect()]                — executes query, returns Arrow RecordBatches
    ↓
[QueryResult]                 — converts RecordBatch → JSON schema + columns
    ↓
HTTP 200 { schema, columns, row_count }
```

### Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `ModelConfiguration` | `semantic_layer/semantic_layer_info/` | Deserializes YAML model (table, metrics, dimensions) |
| `SemanticLayerInfo` | `semantic_layer/` | Registry: model name → `ModelConfiguration` |
| `SemanticLayerContext` | `semantic_layer/` | Executes queries via DataFusion |
| `Query<'a>` | `semantic_layer/query/` | Metrics + dimensions bound to a request lifetime |
| `QueryResult` | `semantic_layer/query_result/` | Serializable result with schema metadata |
| `DataSource` | `data_source/` | Trait: registers tables into a DataFusion `SessionContext` |
| `ParquetDataSource` | `data_source/parquet.rs` | Scans a directory and registers `.parquet` files as tables |

---

## Architectural Gaps & Roadmap

### Priority 1 — Query Filters

**Problem**: `Query` carries only metrics and dimensions. No WHERE, HAVING, ORDER BY, or LIMIT support.

**Plan**: Add a `Filter` type to `semantic-core/src/semantic_layer/query/filter.rs`:

```rust
pub struct Filter {
    pub dimension: String,  // "orders.country"
    pub op: FilterOp,       // Eq | NotEq | In | Lt | Gt | ...
    pub value: FilterValue, // String | Number | List
}
```

Translate filters into DataFusion `Expr` predicates inside `build_dataframe()`. Expose via `QueryRequest.filters: Option<Vec<FilterRequest>>` in the server.

Files: `semantic-core/src/semantic_layer/query/`, `semantic_layer_context.rs`, `execute_query.rs`.

---

### Priority 2 — Structured Error Responses

**Problem**: All errors return HTTP 500 with no body. Clients cannot distinguish validation failures from execution errors.

**Plan**: Return structured JSON errors:

```json
{ "error": "INVALID_METRIC", "message": "Metric 'orders.foo' not found in model 'orders'" }
```

Map `QueryError` (bad input) → 400, `ExecutionQueryError` (runtime) → 500. Introduce a shared `ApiError` type in `semantic-gateway-server/src/web_server/api/error.rs`.

---

### Priority 3 — Test Infrastructure

**Problem**: Only YAML deserialization in `model_configuration.rs` is tested. No tests for query execution, HTTP endpoints, or error handling paths.

**Plan**:
- Unit tests in `semantic_layer_context.rs` using DataFusion `MemTable` (keeps `semantic-core` I/O-free).
- Integration tests in `semantic-gateway-server/tests/api_tests.rs` using `actix_web::test`.
- No new test dependencies needed — DataFusion and Actix already provide the required tooling.

---

## Explicitly Out of Scope

| Topic | Reason |
|-------|--------|
| Multi-model joins | No concrete use case defined yet |
| Authentication | Deployment context not decided |
| Hot-reload of models | Excluded by design (see CLAUDE.md) |
