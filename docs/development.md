# Development

The server is one Rust crate, built with Rust 1.100.0.

```console
export HYPERBILLING_CLIENT_ID=user@example.com
export HYPERBILLING_CLIENT_SECRET="${HYPERBILLING_API_KEY}"
make run
```

Open http://localhost:8080/docs for the documentation page. `make help` lists every target.

## Layout

| Path | Responsibility |
| --- | --- |
| `src/app.rs` | Wiring, listeners, and shutdown |
| `src/config.rs` | Flags and environment variables |
| `src/hyperbilling/` | HyperBilling API client and its errors |
| `src/mcp/` | Tool definitions, arguments, and the MCP handler |
| `src/docs.rs`, `src/docs/` | Documentation page, locale catalogs, and the logo |
| `src/observability/` | Metrics and the health and metrics listeners |
| `charts/hyperbilling-mcp/` | Helm chart |

## Checks

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
make coverage
cargo audit
helm lint --strict charts/hyperbilling-mcp
```

Line coverage stays at 70% or above. CI runs the same checks on every pull request.

## Adding a tool

1. Add the argument struct to `src/mcp/args.rs`. Its `schemars` descriptions become the JSON schema the agent reads.
2. Add the handler and one row to the `tool_surface!` table in `src/mcp/server.rs`.
3. Add the Korean description and one entry per argument to `src/docs/locales/ko.json`. A test fails until they exist.

## Image

```console
make zigbuild
make docker-build
```

`make zigbuild` cross-compiles static musl binaries for both architectures with cargo-zigbuild. The image is `scratch` with the binary and a CA bundle.

## Chart

Every key in `values.yaml` carries a helm-docs comment and is covered by `values.schema.json`. Regenerate the chart README after a values change.

```console
helm-docs --chart-search-root charts/hyperbilling-mcp --template-files=README.md.gotmpl --sort-values-order file
```
