# Authentication

HyperBilling authenticates each request with two headers.

| Header | Value |
| --- | --- |
| `X-Client-Id` | The HyperBilling login email that issued the key |
| `X-Client-Secret` | The Open API key from the HyperBilling console. It expires one year after it is issued |

A key is bound to the account that issued it. Sent with another account's email, it is rejected with 401.

## Caller credentials

A caller that sends both headers on its MCP request is proxied with them, so each caller sees only the linked accounts its own key can read.

## Fallback credential

A caller that sends neither header uses the fallback credential from `HYPERBILLING_CLIENT_ID` and `HYPERBILLING_CLIENT_SECRET`, when one is configured. Both must be set together. Without a fallback the call is refused before it reaches HyperBilling.

With a fallback credential configured, anything that can reach the MCP port can read the billing data behind it. Restrict the port with a NetworkPolicy. The chart renders one through `extraObjects`, and `values.yaml` carries a commented example.

## Key handling

- The HyperBilling 401 response echoes the submitted key, so the server drops that body and returns a fixed message.
- Any other error body has the key redacted before it is returned.
- Request and response bodies are never logged.
- The secret header is marked sensitive, so HTTP debug logging does not print it.
