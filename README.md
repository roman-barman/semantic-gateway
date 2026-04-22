# Semantic gateway

A Semantic Layer / BI Gateway server written in Rust. Query your data using business concepts — metrics and dimensions defined in YAML — instead of writing raw SQL.

## What it does

Semantic gateway sits between your data and your clients. You define **semantic models** in YAML files that map business concepts (revenue, orders count, country) to underlying table columns and aggregations. Clients then query those concepts over a simple HTTP API without knowing anything about the underlying schema. The query engine is backed by [Apache DataFusion](https://datafusion.apache.org/), and data is served from Parquet files.

## Quick start

**Prerequisites:** Rust toolchain (1.75+), Parquet data files.

```bash
git clone <repo-url>
cd semantic-gateway

# Place your Parquet files in data/ and YAML models in models/
# (see "Semantic Model Format" below for the model schema)

cargo run -p semantic-gateway-server
```

Health check:

```bash
curl -s http://localhost:8080/health
```

Run a query:

```bash
curl -s -X POST http://localhost:8080/query/execute \
  -H 'Content-Type: application/json' \
  -d '{
    "metrics": ["orders.revenue", "orders.orders_count"],
    "dimensions": ["orders.country"]
  }'
```

## How it works

```
YAML models  +  Parquet files
       ↓
  POST /query/execute  { metrics, dimensions }
       ↓
  semantic-gateway resolves references → builds GROUP BY query
       ↓
  DataFusion executes against Parquet tables
       ↓
  { schema, columns, row_count }
```

Model files and data files are loaded once at startup. Each query reference uses dot notation: `model_name.metric_or_dimension_name`.

## Semantic Model Format

Place YAML files in `--models-dir` (default: `./models`). The file name without extension becomes the model name.

```yaml
# models/orders.yaml  →  model name: "orders"

table: "orders"           # must match a Parquet file stem in --data-dir

metrics:
  revenue:                # referenced as "orders.revenue"
    title: "Revenue"
    aggregate: "sum"      # sum | count
    field: "amount"       # source column
  orders_count:
    title: "Orders count"
    aggregate: "count"
    field: "order_id"

dimensions:
  country:                # referenced as "orders.country"
    title: "Country"
    field: "country"      # source column to group by
  status:
    title: "Status"
    field: "status"
```

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
    { "name": "revenue",       "value_type": "Float"  },
    { "name": "orders_count",  "value_type": "Int"    },
    { "name": "country",       "value_type": "String" }
  ],
  "columns": {
    "revenue":      [1500.0, 320.0],
    "orders_count": [42, 11],
    "country":      ["GE", "RU"]
  },
  "row_count": 2
}
```

`value_type` values: `String`, `Int`, `UInt`, `Float`, `Unknown`.

## Configuration

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--models-dir` | `./models` | Directory containing YAML model files |
| `--data-dir` | `./data` | Directory containing Parquet data files |

### Config files

Configuration is merged in this order (later sources override earlier ones):

1. `config/base.yaml` — base defaults
2. `config/{APP_ENVIRONMENT}.yaml` — environment overrides (set via `APP_ENVIRONMENT`)
3. `APP_*` environment variables — highest priority

**Defaults:**

```yaml
server:
  host: 127.0.0.1
  port: 8080
  log_level: info
```

**Example env override:**

```bash
APP_SERVER__PORT=9090 cargo run -p semantic-gateway-server
```

## Development

```bash
# Build
cargo build
cargo build --release

# Test
cargo test

# Lint & format
cargo clippy
cargo fmt
```

## Project structure

| Crate | Path | Role |
|-------|------|------|
| `semantic-core` | `semantic-core/` | Data source abstraction, semantic model types, DataFusion query engine |
| `semantic-gateway-server` | `semantic-gateway-server/` | Actix-web HTTP server, configuration loading, startup wiring |

## License

MIT — see [LICENSE](LICENSE).
