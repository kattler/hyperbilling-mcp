FROM --platform=$BUILDPLATFORM alpine:3.23 AS certs

FROM scratch

LABEL org.opencontainers.image.title="hyperbilling-mcp" \
      org.opencontainers.image.description="Model Context Protocol server for the Megazone HyperBilling Open API" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.base.name="scratch" \
      org.opencontainers.image.deprecated="false"

# rustls verifies the HyperBilling API against this bundle
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

ARG TARGETARCH
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --chmod=755 hyperbilling-mcp-linux-${TARGETARCH} /app/hyperbilling-mcp
EXPOSE 8080 8081
USER 65532:65532
ENTRYPOINT ["/app/hyperbilling-mcp"]
