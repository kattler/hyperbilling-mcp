import { useEffect, useMemo, useRef, useState } from "react";

import { Anchor } from "./Anchor";
import { Architecture } from "./Architecture";
import { Authentication, Connect, Endpoints, Errors, Limits } from "./Guide";
import { I18nContext, LOCALES, createI18n, useI18n } from "./i18n";
import { Logo } from "./Logo";
import { Tools } from "./Tools";
import type { DocsData, Locale } from "./types";

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
  return (
    <nav aria-label={t("nav.label")}>
      <h2>{t("nav.guide")}</h2>
      <a href="#overview">{t("nav.overview")}</a>
      <a href="#architecture">{t("nav.architecture")}</a>
      <a href="#connect">{t("nav.connect")}</a>
      <a href="#authentication">{t("nav.authentication")}</a>
      <a href="#limits">{t("nav.limits")}</a>
      <a href="#errors">{t("nav.errors")}</a>
      <a href="#endpoints">{t("nav.endpoints")}</a>
      <h2>{t("nav.tools")}</h2>
      {data.tools.map((tool) => (
        <a className="mono" key={tool.name} href={`#${tool.name}`}>
          {tool.name.replace(/^hyperbilling_/, "")}
        </a>
      ))}
    </nav>
  );
}

function Page({ data, onLocale }: { data: DocsData; onLocale: (locale: Locale) => void }) {
  const { locale, t } = useI18n();

  useEffect(() => {
    document.documentElement.lang = locale;
    document.title = t("title");
  }, [locale, t]);

  return (
    <>
      <div className="top">
        <div className="bar">
          <Logo />
          <span className="sep" aria-hidden="true">
            /
          </span>
          <span className="brand">hyperbilling-mcp</span>
          <span className="sep docs" aria-hidden="true">
            /
          </span>
          <span className="muted">{t("docs")}</span>
          <span className="end">
            <span className="pill">v{data.version}</span>
            <LanguageSwitcher onChange={onLocale} />
          </span>
        </div>
      </div>
      <div className="shell">
        <Navigation data={data} />
        <main>
          <div className="hero" id="overview">
            <h1>
              {t("hero.title")}
              <Anchor id="overview" />
            </h1>
            <p>{t("hero.body")}</p>
          </div>
          <Architecture toolCount={data.tools.length} />
          <Connect />
          <Authentication />
          <Limits />
          <Errors />
          <Endpoints />
          <Tools tools={data.tools} />
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
