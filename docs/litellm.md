# LiteLLM

Register the server on a LiteLLM proxy to give every model and client behind the proxy the same billing tools, with LiteLLM handling keys, access control, and usage logs.

```yaml
mcp_servers:
  hyperbilling:
    url: "http://hyperbilling-mcp.kagent:8080/mcp"
    transport: "http"
    extra_headers: ["X-Client-Id", "X-Client-Secret"]
```

The `http` transport is streamable HTTP.

## Endpoint and tool names

Clients connect to `/hyperbilling/mcp` on the proxy, or to `/mcp/` for every registered server at once. LiteLLM prefixes each tool with the server name, so the search tool becomes `hyperbilling-hyperbilling_search_billing`.

## Credentials

Pick one of two ways.

- Configure the [fallback credential](authentication.md#fallback-credential) on this server and send nothing from LiteLLM. Every caller behind the proxy then shares one HyperBilling account.
- Let each caller send its own credential as `x-mcp-hyperbilling-x-client-id` and `x-mcp-hyperbilling-x-client-secret`. LiteLLM forwards them as the two HyperBilling headers.
