import { Anchor } from "./Anchor";
import { Rich, useI18n } from "./i18n";
import { Logo } from "./Logo";
import { Snippet } from "./Snippet";

const LITELLM_CONFIG = `mcp_servers:
  hyperbilling:
    url: "http://hyperbilling-mcp.example.com:8080/mcp"
    transport: "http"
    extra_headers: ["X-Client-Id", "X-Client-Secret"]`;

function Link({ label, dots }: { label: string; dots: string[] }) {
  return (
    <div className="link" aria-hidden="true">
      <span className="label">{label}</span>
      {dots.map((dot) => (
        <i key={dot} className={dot} />
      ))}
    </div>
  );
}

function ServerNode({ toolCount }: { toolCount: number }) {
  const { t } = useI18n();
  return (
    <div className="node self">
      <span className="kind">{t("flow.server.kind")}</span>
      <strong>hyperbilling-mcp</strong>
      <span className="detail">{t("flow.server.detail", { tool_count: toolCount })}</span>
    </div>
  );
}

function ApiNode() {
  const { t } = useI18n();
  return (
    <div className="node">
      <span className="kind">{t("flow.api.kind")}</span>
      <strong className="api">
        <Logo />
        <span>Open API</span>
      </strong>
      <span className="detail">{t("flow.api.detail")}</span>
    </div>
  );
}

export function Architecture({ toolCount }: { toolCount: number }) {
  const { t } = useI18n();
  return (
    <section id="architecture">
      <h2>
        {t("arch.title")}
        <Anchor id="architecture" />
      </h2>
      <details className="arch" id="arch-direct">
        <summary>
          <span className="head">
            <strong>{t("arch.direct")}</strong>
            <span>{t("arch.direct.hint")}</span>
          </span>
        </summary>
        <div className="body">
          <div className="flow" role="img" aria-label={t("flow.label")}>
            <div className="node">
              <span className="kind">{t("flow.agent.kind")}</span>
              <strong>{t("flow.agent")}</strong>
              <span className="detail">{t("flow.agent.detail")}</span>
            </div>
            <Link label={t("flow.link.mcp")} dots={["", "back d3"]} />
            <ServerNode toolCount={toolCount} />
            <Link label={t("flow.link.api")} dots={["d1", "back d2"]} />
            <ApiNode />
          </div>
          <ol className="steps">
            <li>{t("flow.step1")}</li>
            <li>{t("flow.step2")}</li>
            <li>{t("flow.step3")}</li>
            <li>{t("flow.step4")}</li>
          </ol>
        </div>
      </details>
      <details className="arch" id="arch-litellm">
        <summary>
          <span className="head">
            <strong>{t("litellm.title")}</strong>
            <span>{t("arch.litellm.hint")}</span>
          </span>
        </summary>
        <div className="body">
          <p>{t("litellm.body")}</p>
          <div className="flow four" role="img" aria-label={t("litellm.label")}>
            <div className="node">
              <span className="kind">{t("litellm.clients.kind")}</span>
              <strong>{t("litellm.clients")}</strong>
              <span className="detail">{t("litellm.clients.detail")}</span>
            </div>
            <Link label={t("litellm.link.proxy")} dots={["", "back d5"]} />
            <div className="node">
              <span className="kind">{t("litellm.proxy.kind")}</span>
              <strong>LiteLLM</strong>
              <span className="detail">{t("litellm.proxy.detail")}</span>
            </div>
            <Link label={t("flow.link.mcp")} dots={["d1", "back d4"]} />
            <ServerNode toolCount={toolCount} />
            <Link label={t("flow.link.api")} dots={["d2", "back d3"]} />
            <ApiNode />
          </div>
          <p className="after-flow">
            <Rich text={t("litellm.config")} />
          </p>
          <Snippet text={LITELLM_CONFIG} />
          <p>
            <Rich text={t("litellm.endpoint")} />
          </p>
          <p>
            <Rich text={t("litellm.credentials")} />
          </p>
        </div>
      </details>
    </section>
  );
}
