# hyperbilling-mcp

![Version: 0.1.1](https://img.shields.io/badge/Version-0.1.1-informational?style=flat-square) ![Type: application](https://img.shields.io/badge/Type-application-informational?style=flat-square) ![AppVersion: 0.1.1](https://img.shields.io/badge/AppVersion-0.1.1-informational?style=flat-square)

Model Context Protocol server for the Megazone HyperBilling Open API

**Homepage:** <https://github.com/kattler/hyperbilling-mcp>

## Installation

### List available versions

This chart is distributed via OCI registry, so you need to use [crane](https://github.com/google/go-containerregistry/blob/main/cmd/crane/README.md) instead of `helm search repo` to discover available versions:

```console
crane ls ghcr.io/kattler/charts/hyperbilling-mcp
```

If you need to install crane on macOS, you can easily install it using [Homebrew](https://brew.sh/), the package manager.

```bash
brew install crane
```

### Install the chart

Install the chart with the release name `hyperbilling-mcp`:

```console
helm install hyperbilling-mcp oci://ghcr.io/kattler/charts/hyperbilling-mcp
```

Install with custom values:

```console
helm install hyperbilling-mcp oci://ghcr.io/kattler/charts/hyperbilling-mcp -f values.yaml
```

Install a specific version:

```console
helm install hyperbilling-mcp oci://ghcr.io/kattler/charts/hyperbilling-mcp --version 0.1.1
```

### Install from local chart

Download hyperbilling-mcp chart and install from local directory:

```console
helm pull oci://ghcr.io/kattler/charts/hyperbilling-mcp --untar --version 0.1.1
helm install hyperbilling-mcp ./hyperbilling-mcp
```

The `--untar` option downloads and unpacks the chart files into a directory for easy viewing and editing.

## Upgrade

```console
helm upgrade hyperbilling-mcp oci://ghcr.io/kattler/charts/hyperbilling-mcp
```

## Uninstall

```console
helm uninstall hyperbilling-mcp
```

## Configuration

The following table lists the configurable parameters and their default values.

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| affinity | object | `{}` | Affinity |
| docs.enabled | bool | `true` | Serve the documentation page at `/docs` on the http port |
| extraEnv | list | `[]` | Extra container env entries |
| extraEnvFrom | list | `[]` | Extra container envFrom entries |
| extraObjects | list | `[]` | Extra manifests deployed with the release. Values are rendered through `tpl`, so they can use chart helpers and `.Values` |
| extraPodSpec | object | `{}` | Extra Pod spec fields the chart does not set, merged into the Pod spec as is |
| fullnameOverride | string | `""` | Override the full release name |
| httpRoute.annotations | object | `{}` | HTTPRoute annotations |
| httpRoute.enabled | bool | `false` | Create an HTTPRoute. Skipped when the cluster lacks the Gateway API |
| httpRoute.filters | list | `[]` | Filters applied to matched requests |
| httpRoute.hostnames | list | `[]` | Hostnames the route matches |
| httpRoute.labels | object | `{}` | Extra HTTPRoute labels |
| httpRoute.matches | list | `[{"path":{"type":"PathPrefix","value":"/"}}]` | Request matches sent to the http port, which serves only `/mcp` and `/docs` |
| httpRoute.parentRefs | list | `[]` | Gateways the route attaches to |
| hyperbilling.clientId | string | `""` | Fallback `X-Client-Id` (the HyperBilling login email that issued the key), used when a caller sends no credential headers. Empty serves only callers that send their own |
| hyperbilling.clientSecret | string | `""` | Fallback Open API key, used when `hyperbilling.existingSecret` is empty |
| hyperbilling.existingSecret | string | `""` | Existing Secret with the fallback Open API key. Empty creates one from `hyperbilling.clientSecret` |
| hyperbilling.existingSecretKey | string | `"HYPERBILLING_CLIENT_SECRET"` | Key of the Open API key in the Secret |
| hyperbilling.requestTimeoutSeconds | int | `60` | Timeout of one HyperBilling API request in seconds |
| hyperbilling.url | string | `"https://open.hyperapi.megazone.com"` | HyperBilling Open API base URL |
| image.digest | string | `""` | Image digest (`sha256:...`), pinned as `tag@digest` when set |
| image.pullPolicy | string | `"IfNotPresent"` | Image pull policy |
| image.registry | string | `"ghcr.io"` | Image registry host |
| image.repository | string | `"kattler/hyperbilling-mcp"` | Image repository without the registry |
| image.tag | string | `""` | Image tag. Empty uses the chart appVersion |
| imagePullSecrets | list | `[]` | Image pull secrets |
| ingress.annotations | object | `{}` | Ingress annotations |
| ingress.className | string | `""` | IngressClass name. Empty uses the cluster default |
| ingress.enabled | bool | `false` | Create an Ingress |
| ingress.hosts | list | `[{"host":"hyperbilling-mcp.example.com","paths":[{"path":"/","pathType":"Prefix"}]}]` | Hosts and their paths sent to the http port, which serves only `/mcp` and `/docs` |
| ingress.labels | object | `{}` | Extra Ingress labels |
| ingress.tls | list | `[]` | TLS hosts and their certificate Secrets |
| log.format | string | `"json"` | Log format: json or text |
| log.level | string | `"info"` | Log level: trace, debug, info, warn, error |
| nameOverride | string | `""` | Override the chart name |
| nodeSelector | object | `{}` | Node selector |
| podAnnotations | object | `{}` | Extra Pod annotations |
| podLabels | object | `{}` | Extra Pod labels |
| podSecurityContext | object | `{"runAsGroup":65532,"runAsNonRoot":true,"runAsUser":65532,"seccompProfile":{"type":"RuntimeDefault"}}` | Pod security context |
| ports.http | int | `8080` | Port for `/mcp` and `/docs`, the only port a route reaches |
| ports.metrics | int | `8081` | Port for `/metrics`, `/healthz`, and `/readyz`, never routed |
| replicaCount | int | `1` | Replicas. MCP sessions live in Pod memory, so keep one unless the route has session affinity |
| resizePolicy | list | `[{"resourceName":"cpu","restartPolicy":"NotRequired"},{"resourceName":"memory","restartPolicy":"RestartContainer"}]` | Container resize policy for in-place resizing. Empty omits it |
| resources | object | `{"limits":{"memory":"64Mi"},"requests":{"cpu":"10m","memory":"32Mi"}}` | Container resources |
| revisionHistoryLimit | int | `3` | Old ReplicaSets kept for rollback |
| securityContext | object | `{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"readOnlyRootFilesystem":true}` | Container security context |
| service.trafficDistribution | string | `""` | Service `trafficDistribution`, e.g. PreferClose. Empty omits it |
| service.type | string | `"ClusterIP"` | Service type |
| serviceAccount.annotations | object | `{}` | ServiceAccount annotations |
| serviceAccount.automountServiceAccountToken | bool | `false` | Automount the ServiceAccount token |
| serviceAccount.create | bool | `true` | Create a ServiceAccount |
| serviceAccount.name | string | `""` | ServiceAccount name. Empty uses the full name |
| serviceMonitor.enabled | bool | `false` | Create a ServiceMonitor |
| serviceMonitor.honorLabels | bool | `false` | Keep the metric's labels when they collide with the target's |
| serviceMonitor.interval | string | `"30s"` | Scrape interval |
| serviceMonitor.labels | object | `{}` | Extra ServiceMonitor labels |
| serviceMonitor.metricRelabelings | list | `[]` | Metric relabelings applied before ingestion |
| serviceMonitor.relabelings | list | `[]` | Relabelings applied before scraping |
| serviceMonitor.scrapeTimeout | string | `"10s"` | Scrape timeout |
| strategy | object | `{"rollingUpdate":{"maxSurge":1,"maxUnavailable":0},"type":"RollingUpdate"}` | Deployment update strategy |
| strategy.rollingUpdate.maxSurge | string|int | `1` | Max extra Pods during an update |
| strategy.rollingUpdate.maxUnavailable | string|int | `0` | Max unavailable Pods during an update |
| terminationGracePeriodSeconds | int | `30` | Shutdown grace period in seconds |
| tolerations | list | `[]` | Tolerations |
| topologySpreadConstraints | list | `[]` | Topology spread constraints. A missing `labelSelector` uses the chart's selector labels |

## Source Code

* <https://github.com/kattler/hyperbilling-mcp>

## Maintainers

| Name | Email | Url |
| ---- | ------ | --- |
| younsl | <cysl@kakao.com> | <https://github.com/younsl> |

## License

This chart is licensed under the Apache License 2.0. See the [LICENSE](https://github.com/kattler/hyperbilling-mcp/blob/main/LICENSE) file for details.

## Contributing

This repository does not accept external contributions. Pull requests and issues are disabled.

----------------------------------------------
Autogenerated from chart metadata using [helm-docs v1.14.2](https://github.com/norwoodj/helm-docs/releases/v1.14.2)
