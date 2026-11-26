# Changelog

Notable changes to hyperbilling-mcp, newest first. Entries are headed by the image version and its release date (KST). Chart-only changes are listed under the chart version.

## 0.1.0 (2026-11-26)

First release.

### Added

- Streamable HTTP MCP endpoint at `/mcp` with four read-only tools over the Megazone HyperBilling Open API: `hyperbilling_list_linked_accounts`, `hyperbilling_list_tag_keys`, `hyperbilling_search_billing`, and `hyperbilling_get_recent_billing`.
- Region and service code filters, tag grouping, and pagination through `NextDataToken` on `hyperbilling_search_billing`.
- Seven fixed recent reports on `hyperbilling_get_recent_billing`.
- Caller credentials from the `X-Client-Id` and `X-Client-Secret` request headers, with an optional fallback credential from the environment.
- Array and object arguments accepted as strings, for MCP clients that send every argument as a string.
- Documentation page at `/docs` in English and Korean, rendered from the registered tool definitions. The theme and the language follow the browser, and the layout is responsive.
- Animated architecture diagrams on the documentation page for a direct MCP client and for a LiteLLM proxy in front.
- Prometheus metrics on a separate port for tool calls, tool latency, and API requests.
- `/healthz` and `/readyz` probes, and graceful shutdown on SIGTERM.

### Security

- The HyperBilling 401 response body is dropped because it echoes the submitted key, and the key is redacted from every other error body.
- Scratch image holding the static binary and a CA bundle, running as a non-root user.
- No advisory against any crate in the lockfile at release time.

### Chart 0.1.0

- Deployment, Service, ServiceAccount, optional Secret, and optional ServiceMonitor.
- `values.schema.json` that rejects unknown keys and wrong types.
- `extraObjects` rendered through `tpl`, with a NetworkPolicy example.
- In-place resize policy: a CPU change does not restart the container, a memory change does.
