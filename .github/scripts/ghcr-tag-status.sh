#!/usr/bin/env bash
# Usage: ghcr-tag-status.sh <owner/image> <tag>   Env: GH_TOKEN, GITHUB_ACTOR
# Prints "exists" or "new". A failed lookup exits non-zero, never "new".
set -euo pipefail

IMAGE="${1:?Usage: ghcr-tag-status.sh <owner/image> <tag>}"
TAG="${2:?}"
REGISTRY="${REGISTRY:-ghcr.io}"

TOKEN=$(curl -sS -u "${GITHUB_ACTOR}:${GH_TOKEN}" \
  "https://${REGISTRY}/token?service=${REGISTRY}&scope=repository:${IMAGE}:pull" \
  | jq -r '.token // empty')
if [[ -z "${TOKEN}" ]]; then
  echo "::error::GHCR refused a pull token for ${IMAGE}" >&2
  exit 1
fi

BODY=$(mktemp)
HTTP=$(curl -sS -o "${BODY}" -w '%{http_code}' \
  -H "Authorization: Bearer ${TOKEN}" \
  "https://${REGISTRY}/v2/${IMAGE}/tags/list")

case "${HTTP}" in
  200)
    if jq -e --arg v "${TAG}" '(.tags // []) | index($v)' "${BODY}" > /dev/null; then
      echo "exists"
    else
      echo "new"
    fi
    ;;
  404) echo "new" ;;
  *)
    echo "::error::GHCR lookup for ${IMAGE} failed (HTTP ${HTTP})" >&2
    exit 1
    ;;
esac
