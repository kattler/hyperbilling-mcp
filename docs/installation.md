# Installation

The image and the chart are published to GHCR for `linux/amd64` and `linux/arm64`.

```console
crane ls ghcr.io/kattler/hyperbilling-mcp
crane ls ghcr.io/kattler/charts/hyperbilling-mcp
```

## Helm

Create a Secret holding the Open API key first, then install the chart.

```console
kubectl create secret generic hyperbilling-mcp \
  --namespace kagent \
  --from-literal=HYPERBILLING_CLIENT_SECRET="${HYPERBILLING_API_KEY}"

helm install hyperbilling-mcp oci://ghcr.io/kattler/charts/hyperbilling-mcp \
  --namespace kagent \
  --set hyperbilling.clientId=user@example.com \
  --set hyperbilling.existingSecret=hyperbilling-mcp
```

Leave `hyperbilling.clientId` and the Secret out to serve only callers that send their own credential headers. Every chart value is described in the [chart README](../charts/hyperbilling-mcp/README.md).

MCP sessions live in Pod memory, so run one replica unless the route in front has session affinity.

## Expose outside the cluster

The Service is `ClusterIP`. To reach the server from outside the cluster, enable an HTTPRoute or an Ingress. Both route only `/mcp` by default, so `/docs` stays inside the cluster unless you add it.

```console
helm upgrade hyperbilling-mcp oci://ghcr.io/kattler/charts/hyperbilling-mcp \
  --namespace kagent \
  --reuse-values \
  --set httpRoute.enabled=true \
  --set 'httpRoute.parentRefs[0].name=external' \
  --set 'httpRoute.parentRefs[0].namespace=gateway-system' \
  --set 'httpRoute.hostnames[0]=hyperbilling-mcp.example.com'
```

For an Ingress, set `ingress.enabled`, `ingress.className`, and `ingress.hosts` instead.

The server has no authentication of its own. Do not expose it with a [fallback credential](authentication.md#fallback-credential) configured unless the gateway authenticates callers, because anyone who reaches the route then reads the billing data without credentials.

## Container

```console
docker run --rm -p 8080:8080 -p 8081:8081 \
  -e HYPERBILLING_CLIENT_ID=user@example.com \
  -e HYPERBILLING_CLIENT_SECRET="${HYPERBILLING_API_KEY}" \
  ghcr.io/kattler/hyperbilling-mcp:0.1.0
```

The image is `scratch` with a static binary and a CA bundle. It runs as UID 65532 with a read-only root filesystem and no capabilities.

## Connect a client

The server speaks streamable HTTP at `/mcp`.

```json
{
  "mcpServers": {
    "hyperbilling": {
      "type": "http",
      "url": "http://hyperbilling-mcp.kagent:8080/mcp",
      "headers": {
        "X-Client-Id": "user@example.com",
        "X-Client-Secret": "${HYPERBILLING_API_KEY}"
      }
    }
  }
}
```

To put a LiteLLM proxy in front, see [LiteLLM](litellm.md).
