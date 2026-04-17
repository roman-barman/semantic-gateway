# Architecture

## Overview

`semantic-gateway` is a proof-of-concept semantic layer. The codebase is clean and compiles without warnings, but several gaps block production use. This document describes the current architecture and the planned improvements, ordered by priority.

---

## Current Architecture

### Crate Structure

```
semantic-gateway/
├── semantic-core/          # I/O-free query engine and semantic model
│   ├── semantic_configuration/   # YAML model deserialization
│   └── semantic_layer/           # Query execution via DataFusion
└── semantic-gateway-server/      # HTTP server, config, model loading
    ├── infrastructure/           # Model reader, tracing setup
    └── web_server/api/           # /health, /query/execute handlers
```

### Request Flow

```
POST /query/execute { metrics, dimensions }
    ↓
[execute_query handler]   — parses "model.field" references, validates format
    ↓
[SemanticLayerContext]    — builds DataFusion DataFrame (group_by + aggregate)
    ↓
[QueryResult]             — serializes RecordBatch → JSON schema + columns
    ↓
HTTP 200 { schema, columns, row_count }
```

### Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `ModelConfiguration` | `semantic_configuration/` | Deserializes YAML model (table, metrics, dimensions) |
| `SemanticLayerInfo` | `semantic_layer/` | Registry: model name → `ModelConfiguration` |
| `SemanticLayerContext` | `semantic_layer/` | Executes queries via DataFusion |
| `Query<'a>` | `semantic_layer/query/` | Metrics + dimensions bound to a request lifetime |
| `QueryResult` | `semantic_layer/query_result/` | Serializable result with schema metadata |

---

## Architectural Gaps & Roadmap

### Priority 1 — Data Source Abstraction (Critical)

**Problem**: `SemanticLayerContext::create_orders_table()` hardcodes 6 rows of fake data. The server cannot query real data.

**Plan**: Introduce a `DataSource` trait in `semantic-core`:

```rust
pub trait DataSource: Send + Sync {
    async fn register(&self, ctx: &SessionContext) -> Result<(), DataSourceError>;
}
```

Provide a `ParquetDataSource` implementation (reads `.parquet` files by table name). `SemanticLayerContext` receives `Arc<dyn DataSource>` instead of constructing data internally.

Files: `semantic-core/src/semantic_layer/semantic_layer_context.rs`, new `semantic-core/src/data_source/` module, `semantic-gateway-server/src/main.rs`.

---

### Priority 2 — Query Filters

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

### Priority 3 — Structured Error Responses

**Problem**: All errors return HTTP 500 with no body. Clients cannot distinguish validation failures from execution errors.

**Plan**: Return structured JSON errors:

```json
{ "error": "INVALID_METRIC", "message": "Metric 'orders.foo' not found in model 'orders'" }
```

Map `QueryError` (bad input) → 400, `ExecutionQueryError` (runtime) → 500. Introduce a shared `ApiError` type in `semantic-gateway-server/src/web_server/api/error.rs`.

---

### Priority 4 — SessionContext Reuse

**Problem**: A new `SessionContext::new()` is created on every HTTP request. This is expensive — DataFusion context setup is non-trivial.

**Plan**: Pre-build the `SessionContext` at startup (with data sources already registered) and share it as `Arc<SessionContext>` via Actix `web::Data<>`. Requests create only query expressions, not the full context.

Files: `semantic_layer_context.rs`, `main.rs`, `web_server.rs`.

---

### Priority 5 — Test Infrastructure

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
