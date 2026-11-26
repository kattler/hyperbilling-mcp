#!/usr/bin/env bash
# Usage: detect-release.sh   Writes version and release to GITHUB_OUTPUT.
# Env: GH_TOKEN, GITHUB_ACTOR, GITHUB_REPOSITORY_OWNER, GITHUB_OUTPUT, FORCE
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OWNER="${GITHUB_REPOSITORY_OWNER,,}"
NAME="hyperbilling-mcp"
FORCE="${FORCE:-false}"

version=$(sed -nE 's/.*org\.opencontainers\.image\.version="([^"]+)".*/\1/p' Dockerfile | head -1)
if [[ -z "${version}" ]]; then
  echo "::error::org.opencontainers.image.version label not found in Dockerfile"
  exit 1
fi

crate_version=$(sed -nE 's/^version = "([^"]+)"$/\1/p' Cargo.toml | head -1)
if [[ "${crate_version}" != "${version}" ]]; then
  echo "::error::Cargo.toml version ${crate_version} differs from the Dockerfile label ${version}"
  exit 1
fi

status=$("${SCRIPT_DIR}/ghcr-tag-status.sh" "${OWNER}/${NAME}" "${version}")
release="true"
if [[ "${status}" == "exists" && "${FORCE}" != "true" ]]; then
  echo "${NAME}:${version} is already published, skipping the image"
  release="false"
else
  echo "${NAME}:${version} will be released (${status}, force=${FORCE})"
fi

{
  echo "version=${version}"
  echo "release=${release}"
} >> "${GITHUB_OUTPUT}"
