# semantic-gateway

A Semantic Layer / BI Gateway server written in Rust. It exposes an HTTP API that allows clients to query data using business concepts — metrics and dimensions — defined in YAML model files, rather than writing raw SQL. The query engine is backed by Apache DataFusion.

---

## Commands

```bash
# Build
cargo build
cargo build --release

# Run (default models directory: ./models)
cargo run -p semantic-gateway-server

# Test
cargo test
cargo test

# Lint & Format
cargo clippy
cargo fmt
```

---

## Configuration

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

## Semantic Model Format

Models are YAML files placed in the models directory. The file name (without extension) becomes the model name.

```yaml
# models/orders.yaml → model name: "orders"
table: "orders"          # underlying table/source name
metrics:
  revenue:               # metric name
    title: "Revenue"
    aggregate: "sum"     # supported: sum, count
    field: "amount"      # source column to aggregate
dimensions:
  country:               # dimension name
    title: "Country"
    field: "country"     # source column to group by
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
    { "name": "orders_count", "value_type": "Integer" },
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

---

## Key Design Decisions
- Use a zero copy approach where possible.
- Write code in a functional style whenever possible.
- **No `unwrap()` or `expect()` in production paths.** Use `thiserror` for error types and propagate errors with `?`.
- **Async via Tokio only.** Do not use blocking calls inside async contexts.
- **Structured logging via `tracing` macros** (`info!`, `error!`, etc.). Never use `println!` for observability.
- **Models load once at startup.** Hot-reload is not supported by design.

---

## Testing Approach

- Unit tests live in the same file as the code under `#[cfg(test)]`.
- Use real YAML strings in deserialization tests — do not construct structs manually.
- Integration tests for HTTP endpoints are not yet implemented but should be added before production readiness.

