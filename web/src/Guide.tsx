import { Anchor } from "./Anchor";
import { Rich, useI18n } from "./i18n";
import { Snippet } from "./Snippet";

export function Connect() {
  const { t } = useI18n();
  const origin = window.location.origin;
  const config = JSON.stringify(
    {
      mcpServers: {
        hyperbilling: {
          type: "http",
          url: `${origin}/mcp`,
          headers: {
            "X-Client-Id": "user@example.com",
            "X-Client-Secret": "${HYPERBILLING_API_KEY}",
          },
        },
      },
    },
    null,
    2,
  );
  return (
    <section id="connect">
      <h2>
        {t("connect.title")}
        <Anchor id="connect" />
      </h2>
      <p>
        <Rich text={t("connect.body")} />
      </p>
      <Snippet text={config} />
      <p>{t("connect.prompt.body")}</p>
      <Snippet text={t("connect.prompt", { origin })} />
      <p>
        <Rich text={t("connect.start")} />
      </p>
    </section>
  );
}

export function Authentication() {
  const { t } = useI18n();
  return (
    <section id="authentication">
      <h2>
        {t("auth.title")}
        <Anchor id="authentication" />
      </h2>
      <p>{t("auth.body")}</p>
      <div className="table">
        <table>
          <thead>
            <tr>
              <th>{t("auth.header")}</th>
              <th>{t("auth.value")}</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <span className="name">X-Client-Id</span>
              </td>
              <td>{t("auth.client_id")}</td>
            </tr>
            <tr>
              <td>
                <span className="name">X-Client-Secret</span>
              </td>
              <td>{t("auth.client_secret")}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p>{t("auth.fallback")}</p>
    </section>
  );
}

export function Limits() {
  const { t } = useI18n();
  const cards = [
    { value: "500", label: t("limits.requests") },
    { value: "1000", label: t("limits.rows") },
    { value: "5", label: t("limits.dimensions") },
    { value: t("limits.daily.value"), label: t("limits.daily") },
    { value: t("limits.monthly.value"), label: t("limits.monthly") },
  ];
  return (
    <section id="limits">
      <h2>
        {t("limits.title")}
        <Anchor id="limits" />
      </h2>
      <p>{t("limits.body")}</p>
      <div className="grid">
        {cards.map((card) => (
          <div className="card" key={card.label}>
            <strong>{card.value}</strong>
            <span>{card.label}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

export function Errors() {
  const { t } = useI18n();
  const rows = [
    { status: "204", meaning: t("errors.204") },
    { status: "400", meaning: t("errors.400") },
    { status: "401", meaning: t("errors.401") },
    { status: "403", meaning: t("errors.403") },
    { status: "429", meaning: t("errors.429") },
  ];
  return (
    <section id="errors">
      <h2>
        {t("errors.title")}
        <Anchor id="errors" />
      </h2>
      <p>{t("errors.body")}</p>
      <div className="table">
        <table>
          <thead>
            <tr>
              <th>{t("errors.status")}</th>
              <th>{t("errors.meaning")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.status}>
                <td>
                  <span className="name">{row.status}</span>
                </td>
                <td>{row.meaning}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

export function Endpoints() {
  const { t } = useI18n();
  const rows = [
    { path: "/mcp", port: "http", purpose: t("endpoints.mcp") },
    { path: "/docs", port: "http", purpose: t("endpoints.docs") },
    { path: "/healthz", port: "http", purpose: t("endpoints.healthz") },
    { path: "/readyz", port: "http", purpose: t("endpoints.readyz") },
    { path: "/metrics", port: "metrics", purpose: t("endpoints.metrics") },
  ];
  return (
    <section id="endpoints">
      <h2>
        {t("endpoints.title")}
        <Anchor id="endpoints" />
      </h2>
      <div className="table">
        <table>
          <thead>
            <tr>
              <th>{t("endpoints.path")}</th>
              <th>{t("endpoints.port")}</th>
              <th>{t("endpoints.purpose")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.path}>
                <td>
                  <span className="name">{row.path}</span>
                </td>
                <td>{row.port}</td>
                <td>{row.purpose}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
