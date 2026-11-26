# Configuration

Every flag falls back to an environment variable, which is how the chart sets it.

| Flag | Environment variable | Default | Description |
| --- | --- | --- | --- |
| `--listen-port` | `LISTEN_PORT` | `8080` | Port for `/mcp` and `/docs` |
| `--metrics-port` | `METRICS_PORT` | `8081` | Port for `/metrics`, `/healthz`, and `/readyz` |
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
| `/docs/api` | http | HyperBilling Open API reference |
| `/docs/status` | http | Recent reachability checks as JSON, read by the status board |
| `/healthz` | metrics | Liveness |
| `/readyz` | metrics | Readiness. `503` until the HyperBilling API has been reached once, `200` from then on |
| `/metrics` | metrics | Prometheus metrics |

The http port carries only what a client may see, so a route can send a whole hostname to it. Probes and metrics stay on the metrics port, which no route reaches.

## Readiness

A pod starts not ready. The server requests `HYPERBILLING_URL` every 2 seconds with a 5 second timeout until one request is answered, then `/readyz` turns to `200` and stays there until the process exits. A new pod that cannot reach the API, because of egress rules, DNS, or TLS trust, never receives traffic, so a rollout stops while the old pods keep serving.

The check sends no credentials, so it spends none of the hourly quota. Any answer below `500` counts as reachable, including `401`, `404`, and `429`. A connection failure, a timeout, or a `5xx` answer does not.

After the pod is ready the check runs every 15 seconds and only sets the `hyperbilling_mcp_upstream_reachable` gauge. An API outage leaves pods ready, so a tool call returns the upstream failure as a tool error and `/docs` stays up. Alert on the gauge to catch an outage. `/healthz` ignores the API, so an outage does not restart pods.

## TLS trust

The server verifies the HyperBilling API against the CA bundle at `SSL_CERT_FILE`, which the image sets to its bundled file. Mount another bundle and point `SSL_CERT_FILE` at it when a TLS-inspecting proxy sits in between.
