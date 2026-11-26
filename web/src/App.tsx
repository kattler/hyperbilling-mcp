import { useEffect, useMemo, useRef, useState } from "react";

import { Anchor } from "./Anchor";
import { API_ENDPOINTS, Api } from "./Api";
import { Architecture } from "./Architecture";
import { Authentication, Connect, Endpoints, Errors, Limits } from "./Guide";
import { I18nContext, LOCALES, createI18n, useI18n } from "./i18n";
import { BrandLogo } from "./Logo";
import { Status } from "./Status";
import { ThemeToggle } from "./Theme";
import { Tools } from "./Tools";
import type { DocsData, Locale, PageName } from "./types";

// Relative, so the pages keep working behind a path prefix.
const ROUTES: Record<PageName, Record<PageName | "status", string>> = {
  guide: { guide: "docs", api: "docs/api", status: "docs/status" },
  api: { guide: "../docs", api: "api", status: "status" },
};

function LanguageSwitcher({ onChange }: { onChange: (locale: Locale) => void }) {
  const { locale, t } = useI18n();
  const menu = useRef<HTMLDetailsElement>(null);
  const current = LOCALES.find((entry) => entry.code === locale);
  return (
    <details className="lang" ref={menu}>
      <summary aria-label={t("ui.language")} lang={locale}>
        {current?.label}
      </summary>
      <div className="menu">
        {LOCALES.map((entry) => (
          <button
            key={entry.code}
            type="button"
            lang={entry.code}
            className={entry.code === locale ? "on" : undefined}
            aria-current={entry.code === locale ? "true" : undefined}
            onClick={() => {
              onChange(entry.code);
              if (menu.current) menu.current.open = false;
            }}
          >
            {entry.label}
          </button>
        ))}
      </div>
    </details>
  );
}

function Navigation({ data }: { data: DocsData }) {
  const { t } = useI18n();
  if (data.page === "api") {
    return (
      <nav aria-label={t("nav.label")}>
        <h2>{t("nav.reference")}</h2>
        <a href="#overview">{t("nav.overview")}</a>
        <a href="#base">{t("nav.api.base")}</a>
        <a href="#endpoints">{t("nav.endpoints")}</a>
        <a href="#filter">{t("nav.api.filter")}</a>
        <a href="#values">{t("nav.api.values")}</a>
        <a href="#response">{t("nav.api.response")}</a>
        <a href="#status-codes">{t("nav.api.status_codes")}</a>
        <a href="#limits">{t("nav.limits")}</a>
        <a href="#regions">{t("nav.api.regions")}</a>
        <h2>
          {t("nav.endpoints")}
          <span className="count">{3 + data.views.length + 1}</span>
        </h2>
        {API_ENDPOINTS.map((endpoint) => (
          <a className="mono" key={endpoint.id} href={`#${endpoint.id}`}>
            {endpoint.path.replace(/^\/v\d\/search\//, "")}
          </a>
        ))}
      </nav>
    );
  }
  return (
    <nav aria-label={t("nav.label")}>
      <h2>{t("nav.guide")}</h2>
      <a href="#overview">{t("nav.overview")}</a>
      <a href="#status">{t("nav.status")}</a>
      <a href="#architecture">{t("nav.architecture")}</a>
      <a href="#connect">{t("nav.connect")}</a>
      <a href="#authentication">{t("nav.authentication")}</a>
      <a href="#limits">{t("nav.limits")}</a>
      <a href="#errors">{t("nav.errors")}</a>
      <a href="#endpoints">{t("nav.endpoints")}</a>
      <h2>
        {t("nav.tools")}
        <span className="count">{data.tools.length}</span>
      </h2>
      {data.tools.map((tool) => (
        <a className="mono" key={tool.name} href={`#${tool.name}`}>
          {tool.name.replace(/^hyperbilling_/, "")}
        </a>
      ))}
    </nav>
  );
}

function Tabs({ page }: { page: PageName }) {
  const { locale, t } = useI18n();
  const tabs: { name: PageName; label: string }[] = [
    { name: "guide", label: t("tab.guide") },
    { name: "api", label: t("tab.api") },
  ];
  return (
    <div className="tabs" role="navigation" aria-label={t("tab.label")}>
      {tabs.map((tab) => (
        <a
          key={tab.name}
          href={`${ROUTES[page][tab.name]}?lang=${locale}`}
          className={tab.name === page ? "on" : undefined}
          aria-current={tab.name === page ? "page" : undefined}
        >
          {tab.label}
        </a>
      ))}
    </div>
  );
}

function Page({ data, onLocale }: { data: DocsData; onLocale: (locale: Locale) => void }) {
  const { locale, t } = useI18n();
  const routes = ROUTES[data.page];

  useEffect(() => {
    document.documentElement.lang = locale;
    document.title = data.page === "api" ? t("api.page_title") : t("title");
  }, [data.page, locale, t]);

  return (
    <>
      <a className="skip" href="#content">
        {t("ui.skip")}
      </a>
      <div className="top">
        <div className="bar">
          <a className="brand" href={`${routes.guide}?lang=${locale}`} aria-label={t("ui.home")}>
            <BrandLogo />
          </a>
          <Tabs page={data.page} />
          <span className="end">
            <span className="pill">v{data.version}</span>
            <ThemeToggle />
            <LanguageSwitcher onChange={onLocale} />
          </span>
        </div>
      </div>
      <div className="shell">
        <Navigation data={data} />
        <main id="content" tabIndex={-1}>
          {data.page === "api" ? (
            <Api data={data} guide={`${routes.guide}?lang=${locale}`} />
          ) : (
            <>
              <div className="hero" id="overview">
                <h1>
                  {t("hero.title")}
                  <Anchor id="overview" />
                </h1>
                <p>{t("hero.body")}</p>
              </div>
              <Status url={routes.status} />
              <Architecture toolCount={data.tools.length} />
              <Connect />
              <Authentication />
              <Limits />
              <Errors />
              <Endpoints />
              <Tools tools={data.tools} />
            </>
          )}
          <footer>
            hyperbilling-mcp v{data.version}
            <span className="built">{t("footer.built", { date: data.built })}</span>
            <br />
            {t("footer.notice")}
          </footer>
        </main>
      </div>
    </>
  );
}

export function App({ data }: { data: DocsData }) {
  const [locale, setLocale] = useState<Locale>(data.locale);
  const i18n = useMemo(() => createI18n(locale), [locale]);

  const onLocale = (next: Locale) => {
    setLocale(next);
    const url = new URL(window.location.href);
    url.searchParams.set("lang", next);
    window.history.replaceState(null, "", url);
  };

  return (
    <I18nContext value={i18n}>
      <Page data={data} onLocale={onLocale} />
    </I18nContext>
  );
}
