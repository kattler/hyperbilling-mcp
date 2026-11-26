# Documentation page

The server embeds its own documentation at `/docs` on the MCP port. Set `DOCS_ENABLED=false` (chart value `docs.enabled`) to turn it off.

## Content

- An architecture section with one expandable panel per setup, each with an animated diagram: a direct MCP client, and a LiteLLM proxy in front.
- Connection, authentication, limits, errors, and endpoints.
- A tool reference rendered at startup from the registered tool definitions, so it always matches what `tools/list` returns.

Every section and tool heading carries a link marker for sharing a direct link.

## Browser detection

The page follows the browser without a setting.

- The black or white theme comes from the operating system color scheme (`prefers-color-scheme`).
- The language comes from the Accept-Language header. The dropdown in the header, or `?lang=en` and `?lang=ko`, overrides it.

The layout is responsive from phone to desktop.

## Translations

The page is fully available in English and Korean. Every string comes from the message catalogs in `src/docs/locales/`. The tests fail when a locale lacks a key, and when the Korean catalog lacks a tool or an argument.

Tool descriptions served to agents over MCP stay English.

## Delivery

The page is one HTML document with an inline stylesheet, no script, and no external asset, so it works in an air-gapped cluster and under a strict Content-Security-Policy. It holds no credential and no billing data.

## Logo

The header and the diagrams show the HyperBilling wordmark, kept in `src/docs/hyperbilling-logo.svg` exactly as MegazoneCloud publishes it. At render time it is cropped to the drawing and painted in the text color so it reads in both themes.
