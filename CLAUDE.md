# semantic-gateway

A Semantic Layer / BI Gateway server written in Rust. It exposes an HTTP API that allows clients to query data using business concepts — metrics and dimensions — defined in YAML model files, rather than writing raw SQL. The query engine is backed by Apache DataFusion.

---

## Codebase Structure

The project is a Cargo workspace with two crates:

| Crate | Path | Purpose |
|-------|------|---------|
| `semantic-core` | `semantic-core/` | Core library: data source abstraction, semantic model types, DataFusion query execution |
| `semantic-gateway-server` | `semantic-gateway-server/` | HTTP server binary: Actix-web routes, configuration loading, startup wiring |

---

## Commands

```bash
# Build
cargo build
cargo build --release

# Run (defaults: --models-dir ./models --data-dir ./data)
cargo run -p semantic-gateway-server

# Run with explicit paths
cargo run -p semantic-gateway-server -- --models-dir ./models --data-dir ./data

# Test
cargo test

# Lint & Format
cargo clippy
cargo fmt
```

---

## Configuration

### CLI Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `--models-dir` | `./models` | Directory containing YAML model files |
| `--data-dir` | `./data` | Directory containing data files (e.g. Parquet) |

### Config Files

Configuration is layered and merged in this order:

1. `config/base.yaml` — base defaults
2. `config/{APP_ENVIRONMENT}.yaml` — environment-specific overrides (selected via `APP_ENVIRONMENT` env var)
3. `APP_*` environment variables — highest priority (double underscore as separator, e.g. `APP_SERVER__PORT=8081`)

**Defaults (`config/base.yaml`):**
```yaml
server:
  host: 127.0.0.1
  port: 8080
  log_level: info
```

---

## Data Sources

At startup the server registers data files from `--data-dir` into DataFusion via the `DataSource` trait. The current implementation is `ParquetDataSource`: it scans the directory for `*.parquet` files and registers each file as a table whose name equals the file stem (e.g. `orders.parquet` → table `orders`).

The `DataSource` trait is defined in `semantic-core`:

```rust
pub trait DataSource: Send + Sync {
    async fn register(&self, ctx: &SessionContext) -> Result<(), DataSourceError>;
}
```

---

## Semantic Model Format

Models are YAML files placed in `--models-dir`. The file name (without extension) becomes the model name.

```yaml
# models/orders.yaml → model name: "orders"
table: "orders"              # underlying table/source name; must match a registered data source
metrics:
  revenue:                   # metric name
    title: "Revenue"
    aggregate: "sum"         # supported: sum, count
    field: "amount"          # source column to aggregate
  orders_count:
    title: "Orders count"
    aggregate: "count"
    field: "order_id"
dimensions:
  country:                   # dimension name
    title: "Country"
    field: "country"         # source column to group by
  status:
    title: "Status"
    field: "status"
```

Query references use dot notation: `model_name.metric_or_dimension_name` (e.g. `orders.revenue`).

---

## API Reference

### `GET /health`
Health check. Returns `200 OK`.

### `POST /query/execute`
Execute a semantic query.

**Request:**
```json
{
  "metrics": ["orders.revenue", "orders.orders_count"],
  "dimensions": ["orders.country"]
}
```

**Response:**
```json
{
  "schema": [
    { "name": "revenue", "value_type": "Float" },
    { "name": "orders_count", "value_type": "Int" },
    { "name": "country", "value_type": "String" }
  ],
  "columns": {
    "revenue": [1500.0, 320.0],
    "orders_count": [42, 11],
    "country": ["GE", "RU"]
  },
  "row_count": 2
}
```

**`value_type` values:** `String`, `Int`, `UInt`, `Float`, `Unknown`

---

## Key Design Decisions
- Use a zero copy approach where possible.
- Write code in a functional style whenever possible.
- **No `unwrap()` or `expect()` in production paths.** Use `thiserror` for error types and propagate errors with `?`.
- **Async via Tokio only.** Do not use blocking calls inside async contexts.
- **Structured logging via `tracing` macros** (`info!`, `error!`, etc.). Never use `println!` for observability.
- **Models and data sources load once at startup.** Hot-reload is not supported by design.

---

## Testing Approach

- Unit tests live in the same file as the code under `#[cfg(test)]`.
- Use real YAML strings in deserialization tests — do not construct structs manually.
- For query execution tests, use an in-memory `MemDataSource` (test-only `DataSource` implementation that registers `MemTable`) rather than real files.
- Integration tests for HTTP endpoints are not yet implemented but should be added before production readiness.
