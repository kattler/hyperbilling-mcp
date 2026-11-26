# Documentation page

The server embeds its own documentation at `/docs` on the MCP port. Set `DOCS_ENABLED=false` (chart value `docs.enabled`) to turn it off.

## Content

- An architecture section with one expandable panel per setup, each with an animated diagram: a direct MCP client, and a LiteLLM proxy in front.
- Connection, authentication, limits, errors, and endpoints.
- A setup prompt in the connection section. Pasted into an MCP client, it has the agent register this server. Its URL is the address the page was opened at.
- A copy button on every code block, shown on hover or focus.
- A tool reference rendered at startup from the registered tool definitions, so it always matches what `tools/list` returns.

Every section and tool heading carries a link marker for sharing a direct link.

## Browser detection

The page follows the browser without a setting.

- The black or white theme comes from the operating system color scheme (`prefers-color-scheme`).
- The language comes from the Accept-Language header. The dropdown in the header, or `?lang=en` and `?lang=ko`, overrides it. The dropdown switches in place without a reload.

The layout is responsive from phone to desktop.

## Translations

The page is fully available in English and Korean. Every string comes from the message catalogs in `web/src/locales/`. The tests fail when a locale lacks a key, when the page never reads a key, and when the Korean catalog lacks a tool or an argument. The type check fails when the page reads a key the English catalog lacks.

Tool descriptions served to agents over MCP stay English.

## Delivery

The page is a React application in `web/`, built with Vite into one script and one stylesheet that the binary embeds and serves at `/docs/app.js` and `/docs/app.css`. The server sends a small HTML shell with the negotiated language and the tool definitions as a JSON data block, and React renders the page from it.

It loads no external asset, so it works in an air-gapped cluster, and its Content-Security-Policy allows neither inline script nor inline style. It holds no credential and no billing data. The page needs JavaScript.

## Logo

The header and the diagrams show the HyperBilling wordmark, kept in `web/src/assets/hyperbilling-logo.svg` exactly as MegazoneCloud publishes it. At render time it is cropped to the drawing and painted in the text color so it reads in both themes.
