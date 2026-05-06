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
6. `SemanticLayerContextFactory::new(layer_info, data_source)` created and stored as `web::Data<SemanticLayerContextFactory>` (shared across requests; holds shared DataFusion `RuntimeEnv`)
7. `WebServer::start()` binds and runs

### Request Flow

```
POST /query/execute { metrics, dimensions, filters? }
    ↓
[execute_query handler]
    ↓
[context_factory.create()]    — creates SemanticLayerContext with shared RuntimeEnv
    ↓
[Query::try_from(&request)]   — validates refs, builds Query with Filter vec
    ↓
[data_source.register()]      — registers Parquet files as DataFusion tables
    ↓
[build_dataframe()]           — builds WHERE (filters) + GROUP BY + aggregate plan
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
| `ModelConfiguration` | `semantic_layer/layer_info/` | Deserializes YAML model (table, metrics, dimensions) |
| `SemanticLayerInfo` | `semantic_layer/` | Registry: model name → `ModelConfiguration` |
| `SemanticLayerContextFactory` | `semantic_layer/context/factory.rs` | Holds shared DataFusion `RuntimeEnv`; creates `SemanticLayerContext` per request |
| `SemanticLayerContext` | `semantic_layer/context.rs` | Executes queries via DataFusion |
| `Query<'a>` | `semantic_layer/query.rs` | Metrics + dimensions + filters bound to a request lifetime |
| `Filter<'a>` | `semantic_layer/filter.rs` | WHERE predicate: field reference, operation, value |
| `FilterOperation` | `semantic_layer/filter.rs` | Enum: `Eq \| Ne \| Lt \| Lte \| Gt \| Gte` |
| `FilterValue<'a>` | `semantic_layer/filter.rs` | Enum: `String \| Int \| Float` |
| `QueryResult` | `semantic_layer/query_result.rs` | Serializable result with schema metadata |
| `DataSource` | `data_source.rs` | Trait: registers tables into a DataFusion `SessionContext` |
| `ParquetDataSource` | `data_source/parquet.rs` | Scans a directory and registers `.parquet` files as tables |

---

## Architectural Gaps & Roadmap

### Priority 1 — Test Infrastructure (partially done)

**Done**: Unit tests for `SemanticLayerContext` (query execution and filter evaluation) live in `semantic-core/src/semantic_layer/context.rs` under `#[cfg(test)]`, using an in-memory `MemDataSource` that registers a `MemTable`. YAML deserialization tests remain in `model_configuration.rs`.

**Remaining**: Integration tests for HTTP endpoints are not yet implemented.

**Plan**:
- Integration tests in `semantic-gateway-server/tests/api_tests.rs` using `actix_web::test`.
- No new test dependencies needed — DataFusion and Actix already provide the required tooling.

---

## Explicitly Out of Scope

| Topic | Reason |
|-------|--------|
| Multi-model joins | No concrete use case defined yet |
| Authentication | Deployment context not decided |
| Hot-reload of models | Excluded by design (see CLAUDE.md) |
