import { Anchor } from "./Anchor";
import { Rich, useI18n } from "./i18n";
import { CopyButton, Value } from "./Snippet";
import type { Tool } from "./types";

function ToolCard({ tool }: { tool: Tool }) {
  const { t, translated } = useI18n();
  return (
    <article className="tool" id={tool.name}>
      <header>
        <h3>
          {tool.name}
          <CopyButton text={tool.name} />
          <Anchor id={tool.name} />
        </h3>
      </header>
      <div className="upstream">
        <span className={`method ${tool.method.toLowerCase()}`}>{tool.method}</span>
        <code>{tool.path}</code>
        <CopyButton text={tool.path} />
      </div>
      <p>{translated(`tool.${tool.name}.description`, tool.description)}</p>
      {tool.arguments.length === 0 ? (
        <p className="none">{t("ui.no_arguments")}</p>
      ) : (
        <div className="table">
          <table>
            <thead>
              <tr>
                <th>{t("ui.argument")}</th>
                <th />
                <th>{t("ui.description")}</th>
              </tr>
            </thead>
            <tbody>
              {tool.arguments.map((argument) => (
                <tr key={argument.name}>
                  <td>
                    <Value text={argument.name} />
                    <br />
                    <span className="type">{argument.kind}</span>
                  </td>
                  <td>
                    <span className={argument.required ? "req" : "opt"}>
                      {argument.required ? t("ui.required") : t("ui.optional")}
                    </span>
                  </td>
                  <td>
                    {translated(`arg.${tool.name}.${argument.name}`, argument.description)}
                    {argument.values.length > 0 && (
                      <div className="values">
                        {argument.values.map((value) => (
                          <code key={value}>{value}</code>
                        ))}
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </article>
  );
}

export function Tools({ tools }: { tools: Tool[] }) {
  const { t } = useI18n();
  return (
    <section id="tools">
      <h2>
        {t("tools.title")}
        <span className="count">{tools.length}</span>
        <Anchor id="tools" />
      </h2>
      <p>
        <Rich text={t("tools.body")} />
      </p>
      {tools.map((tool) => (
        <ToolCard key={tool.name} tool={tool} />
      ))}
    </section>
  );
}
