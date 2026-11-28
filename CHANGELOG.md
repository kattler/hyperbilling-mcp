# Changelog

Notable changes to hyperbilling-mcp, newest first. Entries are headed by the image version and its release date as `YYYY-MM-DD KST`. Chart-only changes are listed under the chart version.

## 0.1.1 (2026-11-28 KST)

### Security

- Links on the documentation page percent-encode the tool name and the language they carry, closing the four findings of the first CodeQL scan.

### Changed

- Built with Rust 1.100.1.

### Chart 0.1.1

- `appVersion` 0.1.1.

## 0.1.0 (2026-11-26 KST)

First release.

### Added

- Streamable HTTP MCP endpoint at `/mcp` with five read-only tools over the Megazone HyperBilling Open API, one for every request of the official guide V1.4.1: `hyperbilling_list_linked_accounts`, `hyperbilling_list_tag_keys`, `hyperbilling_search_billing`, `hyperbilling_get_recent_billing`, and `hyperbilling_get_estimated_cost`.
- Region, service code, tag, operation, usage type, line item type, and product category filters, tag grouping, and pagination through `NextDataToken` on `hyperbilling_search_billing`.
- Seven fixed recent reports on `hyperbilling_get_recent_billing`.
- Caller credentials from the `X-Client-Id` and `X-Client-Secret` request headers, with an optional fallback credential from the environment.
- Array and object arguments accepted as strings, for MCP clients that send every argument as a string.
- Documentation page at `/docs` in English and Korean, a React application embedded in the binary and rendered from the registered tool definitions. The theme and the language follow the browser, and the layout is responsive.
- Setup prompt on the documentation page. It carries the address the page was opened at, so pasting it into an MCP client registers this server.
- Status board on the documentation page for the connection to the HyperBilling API, with the last 60 reachability checks.
- HyperBilling Open API reference at `/docs/api`, written from the official guide V1.4.1 and checked against the live API, with every request, `Filter` field, dimension, measure, status code, and limit.
- Copy button on every code block of the documentation page.
- Build date in the footer of the documentation page.
- Animated architecture diagrams on the documentation page for a direct MCP client and for a LiteLLM proxy in front.
- Prometheus metrics on a separate port for tool calls, tool latency, and API requests. The client-facing port serves only `/mcp` and `/docs`.
- `/healthz` and `/readyz` probes on the metrics port, and graceful shutdown on SIGTERM.
- `/readyz` answers `503` until the pod has reached the HyperBilling API once, so a pod that cannot reach the API never receives traffic. The `hyperbilling_mcp_upstream_reachable` gauge reports the latest check for alerting on an outage.

### Security

- The HyperBilling 401 response body is dropped because it echoes the submitted key, and the key is redacted from every other error body.
- Scratch image holding the static binary and a CA bundle, running as a non-root user.
- No advisory against any crate in the lockfile at release time. The release pipeline fails on any CRITICAL or HIGH vulnerability Trivy finds in the lockfile or the images.

### Chart 0.1.0

- Deployment, Service, ServiceAccount, optional Secret, and optional ServiceMonitor.
- `values.schema.json` that rejects unknown keys and wrong types.
- Optional HTTPRoute and Ingress that route the whole hostname to the client-facing port, both off by default.
- `extraObjects` rendered through `tpl`, with a NetworkPolicy example.
- `extraPodSpec` for Pod spec fields the chart does not set, such as `priorityClassName` and `dnsConfig`.
- In-place resize policy: a CPU change does not restart the container, a memory change does.
