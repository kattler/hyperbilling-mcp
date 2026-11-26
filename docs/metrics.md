# Metrics

Prometheus metrics are served at `/metrics` on their own port (`8081` by default), next to the `/healthz` and `/readyz` probes, so a route to the client-facing port never reaches them. The chart creates a ServiceMonitor with `serviceMonitor.enabled`.

| Metric | Labels | Description |
| --- | --- | --- |
| `hyperbilling_mcp_tool_calls_total` | `tool`, `outcome` | Tool calls by tool and outcome |
| `hyperbilling_mcp_tool_call_duration_seconds` | `tool` | Tool call latency including the API request |
| `hyperbilling_mcp_upstream_requests_total` | `method`, `code` | API requests by status code, `transport_error` when no response arrived |
| `hyperbilling_mcp_upstream_reachable` | none | `1` when the last reachability check reached the API, `0` otherwise |
| `hyperbilling_mcp_build_info` | `version`, `commit` | Build of the running binary |

## Outcomes

| Outcome | Meaning |
| --- | --- |
| `ok` | The API answered and the result was returned |
| `tool_error` | The API rejected the request, or no credential was available |
| `error` | The arguments were invalid, the tool does not exist, or no response arrived |

A tool name that is not registered is counted under `unknown`, so a caller cannot grow the label set.

## Useful queries

```promql
sum(rate(hyperbilling_mcp_upstream_requests_total[1h])) * 3600
```

Requests per hour, to compare with the HyperBilling limit of 500.

```promql
sum by (tool) (rate(hyperbilling_mcp_tool_calls_total{outcome!="ok"}[5m]))
```

Failing tool calls by tool.
