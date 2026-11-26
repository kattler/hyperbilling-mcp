# Configuration

Every flag falls back to an environment variable, which is how the chart sets it.

| Flag | Environment variable | Default | Description |
| --- | --- | --- | --- |
| `--listen-port` | `LISTEN_PORT` | `8080` | Port for `/mcp`, `/docs`, `/healthz`, and `/readyz` |
| `--metrics-port` | `METRICS_PORT` | `8081` | Port for `/metrics` |
| `--hyperbilling-url` | `HYPERBILLING_URL` | `https://open.hyperapi.megazone.com` | Open API base URL |
| `--hyperbilling-client-id` | `HYPERBILLING_CLIENT_ID` | empty | Fallback client ID |
| `--hyperbilling-client-secret` | `HYPERBILLING_CLIENT_SECRET` | empty | Fallback Open API key |
| `--request-timeout-seconds` | `HYPERBILLING_REQUEST_TIMEOUT_SECONDS` | `60` | Timeout of one API request |
| `--docs-enabled` | `DOCS_ENABLED` | `true` | Serve the documentation page at `/docs` |
| `--log-level` | `LOG_LEVEL` | `info` | `RUST_LOG` overrides it |
| `--log-format` | `LOG_FORMAT` | `json` | `json` or `text` |

## Endpoints

| Path | Port | Purpose |
| --- | --- | --- |
| `/mcp` | http | MCP over streamable HTTP |
| `/docs` | http | Documentation page. `/` redirects to it |
| `/healthz` | http | Liveness |
| `/readyz` | http | Readiness |
| `/metrics` | metrics | Prometheus metrics |

## TLS trust

The server verifies the HyperBilling API against the CA bundle at `SSL_CERT_FILE`, which the image sets to its bundled file. Mount another bundle and point `SSL_CERT_FILE` at it when a TLS-inspecting proxy sits in between.
