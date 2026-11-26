# Documentation page

The server embeds its own documentation at `/docs` and a HyperBilling Open API reference at `/docs/api` on the MCP port, so a route that exposes the server exposes the page too. Set `DOCS_ENABLED=false` (chart value `docs.enabled`) to turn both off.

## Content

- A status board for the connection to the HyperBilling API.
- An architecture section with one expandable panel per setup, each with an animated diagram: a direct MCP client, and a LiteLLM proxy in front.
- Connection, authentication, limits, errors, and endpoints.
- A setup prompt in the connection section. Pasted into an MCP client, it has the agent register this server. Its URL is the address the page was opened at.
- A copy button on every code block and on every value a reader may paste elsewhere: tool names, request paths, headers, fields, arguments, dimensions, measures, status codes, and region abbreviations. The listed values of an argument or field carry none. It shows on hover or focus.
- A footer with the version and the build date in UTC.
- A tool reference rendered at startup from the registered tool definitions, so it always matches what `tools/list` returns.

Every section and tool heading carries a link marker for sharing a direct link.

## Status board

The board is one chart of the last 60 reachability checks. Each bar is one check: its height is the latency, and a failed check is a full-height red bar. The label above names the newest check, the axis below carries the time of the newest check and, once all 60 slots are filled, of the oldest, and hovering a bar shows its time, latency, and HTTP status. Times are in the browser's time zone and carry its name. The page reads `/docs/status` every 10 seconds while its tab is visible. When a read fails, the chart dims and says when its data was received.

The chart takes keyboard focus, and the arrow keys read one check at a time. A failed check is hatched as well as red, and a change of state is announced to screen readers once.

The data is the same check that gates readiness. The endpoint returns only times, status codes, and latencies, and never the API URL or an error message. With several replicas each request reaches one pod, so the board shows the checks of whichever pod answered.

## API reference

`/docs/api` is a reference for the HyperBilling Open API, written from the official guide (V1.4.1) and checked against the live API. It covers the base URL and headers, all 11 requests with a request and a response example, every `Filter` field, the grouping dimensions and measures, the response format, status codes, limits, and region abbreviations. Each request links to the tool that calls it.

Where the live API behaved differently from the guide, the page says so in the description. Examples use placeholder account IDs and amounts.

The page links to the official guide, which remains the source of truth. When the guide changes, recheck the page and update `GUIDE_VERSION` in `web/src/Api.tsx`.

## Browser detection

The page follows the browser until the reader chooses otherwise.

- The black or white theme comes from the operating system color scheme (`prefers-color-scheme`). The button in the header switches it, and the browser remembers the choice.
- The language comes from the Accept-Language header. The dropdown in the header, or `?lang=en` and `?lang=ko`, overrides it. The dropdown switches in place without a reload.

The layout is responsive from phone to desktop.

## Translations

The page is fully available in English and Korean. Every string comes from the message catalogs in `web/src/locales/`. The tests fail when a locale lacks a key, when the page never reads a key, and when the Korean catalog lacks a tool or an argument. The type check fails when the page reads a key the English catalog lacks.

Tool descriptions served to agents over MCP stay English.

## Delivery

The page is a React application in `web/`, built with Vite into one script and one stylesheet that the binary embeds and serves at `/docs/app.js` and `/docs/app.css`. The server sends a small HTML shell with the page name, the negotiated language, and the tool definitions as a JSON data block, and React renders the page from it. Both pages share the one bundle.

It loads no external asset, so it works in an air-gapped cluster, and its Content-Security-Policy allows neither inline script nor inline style and limits requests to the page's own origin. It holds no credential and no billing data. The page needs JavaScript.

## Logo

The diagrams show the HyperBilling wordmark, kept in `web/src/assets/hyperbilling-logo.svg` exactly as MegazoneCloud publishes it. At render time it is cropped to the drawing and painted in the text color so it reads in both themes.

The header shows the same wordmark followed by "MCP", drawn as vector shapes to the wordmark's cap height and stem weight in `web/src/Logo.tsx`. It is one image and links to the guide page.
