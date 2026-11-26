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

The image release runs the test and coverage gate, cross-compiles both architectures with cargo-zigbuild, and pushes one multi-platform image tagged with the version and `latest`. The chart is pushed after the image it references.

## Re-release

To rebuild a published version, run the Release workflow by hand with `force` enabled. It overwrites the tag.

## Toolchain pins

The build pins Rust 1.100.0, zig 0.15.2, and cargo-zigbuild 0.23.0. Statically linked musl binaries built with zig 0.16.0 grew their memory without bound at runtime, so bump zig only after watching the memory of a rebuilt image stay flat.
