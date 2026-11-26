# hyperbilling-mcp

[![Container](https://img.shields.io/badge/ghcr.io-hyperbilling--mcp-black?style=flat-square&logo=docker&logoColor=white)](https://github.com/kattler/hyperbilling-mcp/pkgs/container/hyperbilling-mcp)
[![Helm Chart](https://img.shields.io/badge/ghcr.io-charts%2Fhyperbilling--mcp-black?style=flat-square&logo=helm&logoColor=white)](https://github.com/kattler/hyperbilling-mcp/pkgs/container/charts%2Fhyperbilling-mcp)
[![Rust](https://img.shields.io/badge/rust-1.100.0-black?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/kattler/hyperbilling-mcp?style=flat-square&color=black)](LICENSE)

A Model Context Protocol server that exposes the Megazone HyperBilling Open API as read-only tools, so an agent can answer AWS cost questions from billing data. One tool call is one HyperBilling request.

## Quick start

```console
helm install hyperbilling-mcp oci://ghcr.io/kattler/charts/hyperbilling-mcp \
  --namespace kagent \
  --set hyperbilling.clientId=user@example.com \
  --set hyperbilling.existingSecret=hyperbilling-mcp
```

The MCP endpoint is `http://hyperbilling-mcp.<namespace>:8080/mcp`, and the server documents itself at `/docs` in English and Korean.

## Documentation

- [Tools](docs/tools.md)
- [Authentication](docs/authentication.md)
- [Installation](docs/installation.md)
- [Configuration](docs/configuration.md)
- [LiteLLM](docs/litellm.md)
- [Documentation page](docs/documentation-page.md)
- [Metrics](docs/metrics.md)
- [Development](docs/development.md)
- [Releasing](docs/releasing.md)
- [Chart values](charts/hyperbilling-mcp/README.md)
- [Changelog](CHANGELOG.md)

## License

Apache License 2.0. See [LICENSE](LICENSE).

This is an unofficial tool, not affiliated with or endorsed by MegazoneCloud. HyperBilling and its logo are trademarks of their owner, used here only to name the service the tools call.
