# Releasing

A release starts on a push to `main` that changes a version value, and skips a version that is already published. No release is tag-based.

| Artifact | Bump this | Published to |
| --- | --- | --- |
| Container image | `org.opencontainers.image.version` label in `Dockerfile` | `ghcr.io/kattler/hyperbilling-mcp` |
| Helm chart | `version` in `charts/hyperbilling-mcp/Chart.yaml` | `ghcr.io/kattler/charts/hyperbilling-mcp` |

## Steps

1. Bump the version in `Cargo.toml` and the `Dockerfile` label together. Set `appVersion` in `Chart.yaml` to the same value and bump the chart `version`.
2. Add the release to `CHANGELOG.md`.
3. Push to `main`.

The image release runs these jobs in order, and a failing job stops the release before anything is pushed.

| Job | Gate |
| --- | --- |
| test | Documentation page type check and build, formatting, clippy, tests, 70% line coverage, and `cargo audit` |
| build | Static musl binaries for both architectures with cargo-zigbuild |
| scan | Trivy on the lockfile and on both built images. Any CRITICAL or HIGH vulnerability fails the release, with or without a fix available |
| image | One multi-platform image tagged with the version and `latest` |
| chart | The chart, pushed after the image it references |

The image is `scratch` with a stripped static binary, so an image scanner sees no package inside it. The lockfile scan is the one that covers the Rust dependencies in `Cargo.lock` and the documentation page dependencies in `web/package-lock.json`.

Third-party actions are pinned to a commit SHA. The scan job holds no registry write permission.

## Re-release

To rebuild a published version, run the Release workflow by hand with `force` enabled. It overwrites the tag.

## Toolchain pins

The build pins Rust 1.100.0, zig 0.15.2, cargo-zigbuild 0.23.0, and Node 24.18.0 in `web/.node-version`. Statically linked musl binaries built with zig 0.16.0 grew their memory without bound at runtime, so bump zig only after watching the memory of a rebuilt image stay flat.
