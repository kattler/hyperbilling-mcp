import { Anchor } from "./Anchor";
import { type MessageKey, Rich, useI18n } from "./i18n";
import { CopyButton, Snippet, Value } from "./Snippet";
import type { DocsData, Tool } from "./types";

const BASE_URL = "https://open.hyperapi.megazone.com";
const GUIDE_URL =
  "https://billing.mzc.megazone.com/user-guide?path=docs%2Fmzc%2Fen%2Fopen-api%2FuserGuide.json";
const GUIDE_VERSION = "V1.4.1";

const PATHS = {
  billing: "/v1/search/billing",
  tag: "/v1/search/tag",
  linkedaccount: "/v1/search/linkedaccount",
  recent: "/v2/search/billing/{view}",
  estimated: "/v2/search/billing/estimated",
};

export const API_ENDPOINTS = [
  { id: "billing", path: PATHS.billing },
  { id: "tag", path: PATHS.tag },
  { id: "linkedaccount", path: PATHS.linkedaccount },
  { id: "recent", path: PATHS.recent },
  { id: "estimated", path: PATHS.estimated },
];

const VIEW_REPORTS: Record<string, MessageKey> = {
  latest: "api.view.latest",
  "last-3-months": "api.view.last_3_months",
  "last-6-months": "api.view.last_6_months",
  "latest-month-daily": "api.view.latest_month_daily",
  "latest-from-last-month-daily": "api.view.latest_from_last_month_daily",
  "latest-by-region": "api.view.latest_by_region",
  "latest-by-product": "api.view.latest_by_product",
};

interface Named {
  name: string;
  description: MessageKey;
}

interface Field extends Named {
  type: string;
  required?: boolean;
  values?: string[];
}

const FILTER_FIELDS: Field[] = [
  { name: "LinkedAccount", type: "string[]", required: true, description: "api.field.linked_account" },
  { name: "Granularity", type: "string", required: true, description: "api.field.granularity" },
  { name: "TimePeriod", type: "object", description: "api.field.time_period" },
  { name: "UseBillingPeriod", type: "boolean", description: "api.field.use_billing_period" },
  { name: "Region", type: "string[]", description: "api.field.region" },
  { name: "ServiceCode", type: "string[]", description: "api.field.service_code" },
  { name: "Tag", type: "object", description: "api.field.tag" },
  { name: "Operation", type: "string[]", description: "api.field.match" },
  { name: "AZ", type: "string[]", description: "api.field.az" },
  { name: "ResourceId", type: "string[]", description: "api.field.match" },
  { name: "InstanceType", type: "string[]", description: "api.field.match" },
  { name: "UsageType", type: "string[]", description: "api.field.match" },
  { name: "LineItemType", type: "string[]", description: "api.field.match" },
  { name: "Description", type: "string[]", description: "api.field.description" },
  { name: "ProductCategory", type: "string[]", description: "api.field.match" },
  { name: "ReservationAccount", type: "string[]", description: "api.field.match" },
  {
    name: "ReservationPurchaseOption",
    type: "string[]",
    description: "api.field.reservation_option",
    values: ["All", "Partial", "No", "Heavy", "Light"],
  },
  {
    name: "ReservationPurchasePeriod",
    type: "string[]",
    description: "api.field.one_of",
    values: ["1yr", "3yr"],
  },
  { name: "SavingsPlansArn", type: "string[]", description: "api.field.match" },
  {
    name: "SavingsPlansTypes",
    type: "string[]",
    description: "api.field.one_of",
    values: ["EC2", "Compute"],
  },
  {
    name: "SavingsPlansPurchasePeriod",
    type: "string[]",
    description: "api.field.one_of",
    values: ["1yr", "3yr"],
  },
  {
    name: "SavingsPlansPurchaseOption",
    type: "string[]",
    description: "api.field.one_of",
    values: ["All", "Partial", "No"],
  },
  { name: "SavingsPlansAccount", type: "string[]", description: "api.field.match" },
  { name: "NextDataToken", type: "string", description: "api.field.next_data_token" },
];

const BODY_FIELDS: Field[] = [
  { name: "Filter", type: "object", required: true, description: "api.field.filter" },
  { name: "GroupBy", type: "string[]", required: true, description: "api.field.group_by" },
  { name: "Result", type: "string[]", required: true, description: "api.field.result" },
];

const RECENT_FIELDS: Field[] = [
  { name: "Filter.LinkedAccount", type: "string[]", required: true, description: "api.field.linked_account" },
  { name: "Filter.NextDataToken", type: "string", description: "api.field.next_data_token" },
];

const DIMENSIONS: Named[] = [
  { name: "USAGE_DATE", description: "api.dim.usage_date" },
  { name: "BILLING_PERIOD_START_DATE", description: "api.dim.billing_period_start_date" },
  { name: "LINKED_ACCOUNT", description: "api.dim.linked_account" },
  { name: "REGION", description: "api.dim.region" },
  { name: "SERVICE_CODE", description: "api.dim.service_code" },
  { name: "PRODUCT_CATEGORY", description: "api.dim.product_category" },
  { name: "USAGE_TYPE", description: "api.dim.usage_type" },
  { name: "OPERATION", description: "api.dim.operation" },
  { name: "INSTANCE_TYPE", description: "api.dim.instance_type" },
  { name: "RESOURCE_ID", description: "api.dim.resource_id" },
  { name: "LINEITEM_TYPE", description: "api.dim.lineitem_type" },
  { name: "DESCRIPTION", description: "api.dim.description" },
  { name: "UNBLENDED_RATE", description: "api.dim.unblended_rate" },
  { name: "RESERVED_PURCHASE_TYPE", description: "api.dim.reserved_purchase_type" },
  { name: "RESERVED_PERIOD", description: "api.dim.reserved_period" },
  { name: "RESERVED_INSTANCE_ID", description: "api.dim.reserved_instance_id" },
  { name: "RESERVED_ACCOUNT", description: "api.dim.reserved_account" },
  { name: "SAVINGS_PLANS_ARN", description: "api.dim.savings_plans_arn" },
  { name: "SAVINGS_PLANS_ACCOUNT", description: "api.dim.savings_plans_account" },
  { name: "SAVINGS_PLANS_TYPE", description: "api.dim.savings_plans_type" },
  { name: "SAVINGS_PLANS_PERIOD", description: "api.dim.savings_plans_period" },
  { name: "SAVINGS_PLANS_PURCHASE_OPTION", description: "api.dim.savings_plans_purchase_option" },
  { name: "TAG:<key>", description: "api.dim.tag" },
];

const MEASURES: Named[] = [
  { name: "USAGE_COST", description: "api.measure.usage_cost" },
  { name: "USAGE_QUANTITY", description: "api.measure.usage_quantity" },
  { name: "SP_EFFECTIVE_COST", description: "api.measure.sp_effective_cost" },
  { name: "SP_TOT_COMMITMENT_TO_DATE", description: "api.measure.sp_tot_commitment_to_date" },
  { name: "SP_USED_COMMITMENT", description: "api.measure.sp_used_commitment" },
];

const ESTIMATED_FIELDS: Named[] = [
  { name: "CurrentMonthCost", description: "api.est.current" },
  { name: "EstimatedCost", description: "api.est.estimated" },
  { name: "LastMonthCost", description: "api.est.last_month" },
  { name: "MonthOverMonthChangePercent", description: "api.est.change" },
  { name: "Last6MonthsCosts", description: "api.est.six_months" },
  { name: "latestBillingDate", description: "api.est.latest_date" },
];

const RESPONSE_FIELDS: Named[] = [
  { name: "Results", description: "api.resp.results" },
  { name: "Results[].GroupBy", description: "api.resp.group_by" },
  { name: "Results[].Value", description: "api.resp.value" },
  { name: "NextDataToken", description: "api.resp.token" },
];

const STATUSES: { status: string; meaning: MessageKey }[] = [
  { status: "200", meaning: "api.status.200" },
  { status: "204", meaning: "api.status.204" },
  { status: "400", meaning: "api.status.400" },
  { status: "401", meaning: "api.status.401" },
  { status: "403", meaning: "api.status.403" },
  { status: "404", meaning: "api.status.404" },
  { status: "429", meaning: "errors.429" },
  { status: "500", meaning: "api.status.500" },
];

const REGIONS: [string, string][] = [
  ["APE1", "Asia Pacific (Hong Kong)"],
  ["APN1", "Asia Pacific (Tokyo)"],
  ["APN2", "Asia Pacific (Seoul)"],
  ["APS1", "Asia Pacific (Singapore)"],
  ["APS2", "Asia Pacific (Sydney)"],
  ["APS3", "Asia Pacific (Mumbai)"],
  ["CAN1", "Canada (Central)"],
  ["CPT", "Africa (Cape Town)"],
  ["EUN1", "Europe (Stockholm)"],
  ["EUC1", "Europe (Frankfurt)"],
  ["EU", "Europe (Ireland)"],
  ["EUW2", "Europe (London)"],
  ["EUW3", "Europe (Paris)"],
  ["MES1", "Middle East (Bahrain)"],
  ["SAE1", "South America (São Paulo)"],
  ["UGW1", "AWS GovCloud (US-West)"],
  ["UGE1", "AWS GovCloud (US-East)"],
  ["USE1", "US East (N. Virginia)"],
  ["USE2", "US East (Ohio)"],
  ["USW1", "US West (N. California)"],
  ["USW2", "US West (Oregon)"],
];

const EDGE_REGIONS: [string, string][] = [
  ["AP", "Hong Kong, Philippines, South Korea, Taiwan, Singapore"],
  ["AU", "Australia"],
  ["CA", "Canada"],
  ["EU", "Europe and Israel"],
  ["IN", "India"],
  ["JP", "Japan"],
  ["ME", "Middle East"],
  ["SA", "South America"],
  ["US", "United States"],
  ["ZA", "South Africa"],
];

const json = (value: unknown) => JSON.stringify(value, null, 2);

const ACCOUNTS_RESPONSE = json({ Results: ["123456789012", "210987654321"] });
const TAG_REQUEST = json({ LinkedAccount: ["123456789012"] });
const TAG_RESPONSE = json({
  Results: [{ LinkedAccount: "123456789012", Tags: ["user:Team", "aws:eks:namespace"] }],
});
const BILLING_REQUEST = json({
  Filter: {
    LinkedAccount: ["123456789012"],
    Granularity: "MONTHLY",
    TimePeriod: { Start: "2026-10-01", End: "2026-10-31" },
    Region: ["APN2"],
    Tag: { Key: "user:Team", Values: ["platform"] },
  },
  GroupBy: ["USAGE_DATE", "SERVICE_CODE"],
  Result: ["USAGE_COST", "USAGE_QUANTITY"],
});
const BILLING_RESPONSE = json({
  NextDataToken: "",
  Results: [
    {
      GroupBy: { USAGE_DATE: "2026-10", SERVICE_CODE: "AmazonEC2" },
      Value: { USAGE_COST: 412.57, USAGE_QUANTITY: 8930.25 },
    },
    {
      GroupBy: { USAGE_DATE: "2026-10", SERVICE_CODE: "AmazonS3" },
      Value: { USAGE_COST: 25.8, USAGE_QUANTITY: 17751.82 },
    },
  ],
});
const RECENT_REQUEST = json({ Filter: { LinkedAccount: ["123456789012"] } });
const RECENT_RESPONSE = json({
  NextDataToken: "",
  Results: [
    { GroupBy: { USAGE_DATE: "2026-09" }, Value: { USAGE_COST: 106.21 } },
    { GroupBy: { USAGE_DATE: "2026-10" }, Value: { USAGE_COST: 108.0 } },
    { GroupBy: { USAGE_DATE: "2026-11" }, Value: { USAGE_COST: 74.1 } },
  ],
});
const ESTIMATED_RESPONSE = json({
  CurrentMonthCost: 970.96,
  EstimatedCost: 1621.39,
  LastMonthCost: 1694.43,
  MonthOverMonthChangePercent: -4.31,
  Last6MonthsCosts: [
    { Month: "2026-03", Cost: 1694.43 },
    { Month: "2026-04", Cost: 970.96 },
  ],
  latestBillingDate: "2026-04-19",
});
const ERROR_BODY = json({
  timestamp: "2026-11-26T15:36:25.521+0000",
  status: 400,
  error: "Bad Request",
  message: "Field 'GroupBy' elements can not exceed 5.",
  path: PATHS.billing,
});
const CURL = `curl -s ${BASE_URL}${PATHS.billing} \\
  -H "Content-Type: application/json" \\
  -H "X-Client-Id: user@example.com" \\
  -H "X-Client-Secret: \${HYPERBILLING_API_KEY}" \\
  -d '${JSON.stringify(JSON.parse(BILLING_REQUEST))}'`;

function Endpoint({
  id,
  method,
  path,
  body,
  tool,
  guide,
  children,
}: {
  id: string;
  method: "GET" | "POST";
  path: string;
  body: MessageKey;
  tool?: Tool;
  guide: string;
  children: React.ReactNode;
}) {
  const { t } = useI18n();
  return (
    <article className="tool" id={id}>
      <header>
        <span className={`method ${method.toLowerCase()}`}>{method}</span>
        <h3>
          {path}
          <CopyButton text={path} />
          <Anchor id={id} />
        </h3>
      </header>
      <p>
        <Rich text={t(body)} />
      </p>
      <p className="called">
        {t("api.tool")}{" "}
        {tool ? (
          <a className="ref" href={`${guide}#${encodeURIComponent(tool.name)}`}>
            {tool.name}
          </a>
        ) : (
          t("api.tool.none")
        )}
      </p>
      {children}
    </article>
  );
}

function Example({ label, text }: { label: string; text?: string }) {
  const { t } = useI18n();
  return (
    <>
      <h4>{label}</h4>
      {text ? <Snippet text={text} /> : <p className="none">{t("api.no_body")}</p>}
    </>
  );
}

function Fields({ rows }: { rows: Field[] }) {
  const { t } = useI18n();
  return (
    <div className="table">
      <table>
        <thead>
          <tr>
            <th>{t("api.field")}</th>
            <th />
            <th>{t("ui.description")}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.name}>
              <td>
                <Value text={row.name} />
                <br />
                <span className="type">{row.type}</span>
              </td>
              <td>
                <span className={row.required ? "req" : "opt"}>
                  {row.required ? t("ui.required") : t("ui.optional")}
                </span>
              </td>
              <td>
                <Rich text={t(row.description)} />
                {row.values && (
                  <div className="values">
                    {row.values.map((value) => (
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
  );
}

function Pairs({
  head,
  rows,
}: {
  head: [string, string];
  rows: Named[];
}) {
  const { t } = useI18n();
  return (
    <div className="table">
      <table>
        <thead>
          <tr>
            <th>{head[0]}</th>
            <th>{head[1]}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.name}>
              <td>
                <Value text={row.name} />
              </td>
              <td>
                <Rich text={t(row.description)} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Regions({ rows }: { rows: [string, string][] }) {
  const { t } = useI18n();
  return (
    <div className="table">
      <table>
        <thead>
          <tr>
            <th>{t("api.regions.code")}</th>
            <th>{t("api.regions.name")}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map(([code, name]) => (
            <tr key={code}>
              <td>
                <Value text={code} />
              </td>
              <td>{name}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function Api({ data, guide }: { data: DocsData; guide: string }) {
  const { t } = useI18n();
  const toolFor = (path: string) => data.tools.find((tool) => tool.path === path);
  const recentTool = toolFor(PATHS.recent);
  const summary: { method: string; path: string; description: MessageKey; tool?: Tool }[] = [
    {
      method: "POST",
      path: PATHS.billing,
      description: "api.summary.billing",
      tool: toolFor(PATHS.billing),
     
    },
    { method: "POST", path: PATHS.tag, description: "api.summary.tag", tool: toolFor(PATHS.tag) },
    {
      method: "GET",
      path: PATHS.linkedaccount,
      description: "api.summary.linkedaccount",
      tool: toolFor(PATHS.linkedaccount),
     
    },
    ...data.views.map((view) => ({
      method: "POST",
      path: view.path,
      description: VIEW_REPORTS[view.name] ?? "api.view.report",
      tool: recentTool,
     
    })),
    {
      method: "POST",
      path: PATHS.estimated,
      description: "api.summary.estimated",
      tool: toolFor(PATHS.estimated),
    },
  ];
  const limits: { label: string; value: string }[] = [
    { label: t("limits.requests"), value: "2000" },
    { label: t("limits.rows"), value: "1000" },
    { label: t("limits.dimensions"), value: "5" },
    { label: t("limits.daily"), value: t("limits.daily.value") },
    { label: t("limits.monthly"), value: t("limits.monthly.value") },
    { label: t("api.limit.key"), value: t("api.limit.key.value") },
  ];
  return (
    <>
      <div className="hero" id="overview">
        <h1>
          {t("api.title")}
          <Anchor id="overview" />
        </h1>
        <p>{t("api.hero")}</p>
        <a className="source" href={GUIDE_URL} rel="noreferrer">
          <span className="kind">{t("api.source.kind")}</span>
          <strong>
            HyperBilling Open-API <span>{GUIDE_VERSION}</span>
          </strong>
          <span className="detail">{t("api.source.detail")}</span>
          <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            <path d="M5 11L11 5M6 5h5v5" />
          </svg>
        </a>
      </div>

      <section id="base">
        <h2>
          {t("api.base.title")}
          <Anchor id="base" />
        </h2>
        <p>{t("api.base.body")}</p>
        <Snippet text={BASE_URL} />
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
                  <Value text="Content-Type" />
                </td>
                <td>
                  <code>application/json</code>
                </td>
              </tr>
              <tr>
                <td>
                  <Value text="X-Client-Id" />
                </td>
                <td>{t("auth.client_id")}</td>
              </tr>
              <tr>
                <td>
                  <Value text="X-Client-Secret" />
                </td>
                <td>{t("auth.client_secret")}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <p>{t("api.base.key")}</p>
        <Snippet text={CURL} />
      </section>

      <section id="endpoints">
        <h2>
          {t("endpoints.title")}
          <span className="count">{summary.length}</span>
          <Anchor id="endpoints" />
        </h2>
        <p>{t("api.endpoints.body", { count: summary.length })}</p>
        <div className="table">
          <table>
            <thead>
              <tr>
                <th>{t("endpoints.path")}</th>
                <th>{t("ui.description")}</th>
                <th>{t("api.tool.column")}</th>
              </tr>
            </thead>
            <tbody>
              {summary.map((row) => (
                <tr key={row.path}>
                  <td>
                    <span className={`method ${row.method.toLowerCase()}`}>{row.method}</span>{" "}
                    <Value text={row.path} />
                  </td>
                  <td>{t(row.description)}</td>
                  <td>
                    {row.tool ? (
                      <a className="ref" href={`${guide}#${encodeURIComponent(row.tool.name)}`}>
                        {row.tool.name.replace(/^hyperbilling_/, "")}
                      </a>
                    ) : (
                      t("api.tool.none")
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <Endpoint
          id="billing"
          method="POST"
          path={PATHS.billing}
          body="api.billing.body"
          tool={toolFor(PATHS.billing)}
          guide={guide}
        >
          <Fields rows={BODY_FIELDS} />
          <Example label={t("api.request")} text={BILLING_REQUEST} />
          <Example label={t("api.response")} text={BILLING_RESPONSE} />
        </Endpoint>
        <Endpoint
          id="tag"
          method="POST"
          path={PATHS.tag}
          body="api.tag.body"
          tool={toolFor(PATHS.tag)}
          guide={guide}
        >
          <Fields
            rows={[
              {
                name: "LinkedAccount",
                type: "string[]",
                required: true,
               
                description: "api.field.linked_account",
              },
            ]}
          />
          <Example label={t("api.request")} text={TAG_REQUEST} />
          <Example label={t("api.response")} text={TAG_RESPONSE} />
        </Endpoint>
        <Endpoint
          id="linkedaccount"
          method="GET"
          path={PATHS.linkedaccount}
          body="api.linkedaccount.body"
          tool={toolFor(PATHS.linkedaccount)}
          guide={guide}
        >
          <Example label={t("api.request")} />
          <Example label={t("api.response")} text={ACCOUNTS_RESPONSE} />
        </Endpoint>
        <Endpoint
          id="recent"
          method="POST"
          path={PATHS.recent}
          body="api.recent.body"
          tool={recentTool}
          guide={guide}
        >
          <div className="table">
            <table>
              <thead>
                <tr>
                  <th>{t("endpoints.path")}</th>
                  <th>{t("api.view.report")}</th>
                </tr>
              </thead>
              <tbody>
                {data.views.map((view) => {
                  const report = VIEW_REPORTS[view.name];
                  return (
                    <tr key={view.name}>
                      <td>
                        <Value text={view.path} />
                      </td>
                      <td>{report ? t(report) : view.name}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          <p>
            <Rich text={t("api.view.typo")} />
          </p>
          <Fields rows={RECENT_FIELDS} />
          <Example label={t("api.request")} text={RECENT_REQUEST} />
          <Example label={t("api.response")} text={RECENT_RESPONSE} />
        </Endpoint>
        <Endpoint
          id="estimated"
          method="POST"
          path={PATHS.estimated}
          body="api.estimated.body"
          tool={toolFor(PATHS.estimated)}
          guide={guide}
        >
          <Pairs head={[t("api.field"), t("ui.description")]} rows={ESTIMATED_FIELDS} />
          <Example label={t("api.request")} text={RECENT_REQUEST} />
          <Example label={t("api.response")} text={ESTIMATED_RESPONSE} />
        </Endpoint>
      </section>

      <section id="filter">
        <h2>
          {t("api.filter.title")}
          <Anchor id="filter" />
        </h2>
        <p>
          <Rich text={t("api.filter.body")} />
        </p>
        <Fields rows={FILTER_FIELDS} />
        <p>
          <Rich text={t("api.filter.unknown")} />
        </p>
      </section>

      <section id="values">
        <h2>
          {t("api.values.title")}
          <Anchor id="values" />
        </h2>
        <h4>{t("api.dimensions")}</h4>
        <Pairs head={["GroupBy", t("ui.description")]} rows={DIMENSIONS} />
        <h4>{t("api.measures")}</h4>
        <Pairs head={["Result", t("ui.description")]} rows={MEASURES} />
      </section>

      <section id="response">
        <h2>
          {t("api.response.title")}
          <Anchor id="response" />
        </h2>
        <p>{t("api.response.body")}</p>
        <Pairs head={[t("api.field"), t("ui.description")]} rows={RESPONSE_FIELDS} />
      </section>

      <section id="status-codes">
        <h2>
          {t("api.status.title")}
          <Anchor id="status-codes" />
        </h2>
        <div className="table">
          <table>
            <thead>
              <tr>
                <th>{t("errors.status")}</th>
                <th>{t("errors.meaning")}</th>
              </tr>
            </thead>
            <tbody>
              {STATUSES.map((row) => (
                <tr key={row.status}>
                  <td>
                    <Value text={row.status} />
                  </td>
                  <td>{t(row.meaning)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p>{t("api.error.body")}</p>
        <Snippet text={ERROR_BODY} />
      </section>

      <section id="limits">
        <h2>
          {t("limits.title")}
          <Anchor id="limits" />
        </h2>
        <div className="grid">
          {limits.map((limit) => (
            <div className="card" key={limit.label}>
              <strong>{limit.value}</strong>
              <span>
                {limit.label}
              </span>
            </div>
          ))}
        </div>
        <p className="after-grid">
          <Rich text={t("api.limit.end")} />
        </p>
      </section>

      <section id="regions">
        <h2>
          {t("api.regions.title")}
          <Anchor id="regions" />
        </h2>
        <p>
          <Rich text={t("api.regions.body")} />
        </p>
        <Regions rows={REGIONS} />
        <h4>{t("api.regions.edge")}</h4>
        <Regions rows={EDGE_REGIONS} />
      </section>

    </>
  );
}
