import { useEffect, useState } from "react";

import { Anchor } from "./Anchor";
import { useI18n } from "./i18n";

interface Check {
  at: number;
  reachable: boolean;
  status: number | null;
  latency_ms: number;
}

interface Snapshot {
  ready_since: number | null;
  checks: Check[];
}

const REFRESH_MS = 10_000;
const SLOTS = 60;
// A fixed locale keeps the clock as HH:MM:SS, where Korean would spell out the units.
const CLOCK = new Intl.DateTimeFormat("en-GB", {
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
  hour12: false,
});
const SLOT = 10;
const BAR = 8;
const HEIGHT = 100;

function useSnapshot(url: string): { snapshot: Snapshot | null; failed: boolean; received: number } {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [failed, setFailed] = useState(false);
  const [received, setReceived] = useState(0);

  useEffect(() => {
    const abort = new AbortController();
    let timer: number | undefined;
    const load = () => {
      fetch(url, { signal: abort.signal, cache: "no-store" })
        .then((response) => {
          if (!response.ok) throw new Error(String(response.status));
          return response.json() as Promise<Snapshot>;
        })
        .then(
          (next) => {
            setSnapshot(next);
            setReceived(Date.now());
            setFailed(false);
          },
          () => {
            if (!abort.signal.aborted) setFailed(true);
          },
        );
    };
    const stop = () => {
      window.clearInterval(timer);
      timer = undefined;
    };
    const start = () => {
      if (timer !== undefined) return;
      load();
      timer = window.setInterval(load, REFRESH_MS);
    };
    const onVisibility = () => (document.hidden ? stop() : start());

    if (!document.hidden) start();
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      abort.abort();
      stop();
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [url]);

  return { snapshot, failed, received };
}

function ceiling(value: number): number {
  const step = value <= 200 ? 50 : value <= 1000 ? 250 : 1000;
  return Math.max(step, Math.ceil(value / step) * step);
}

export function Status({ url }: { url: string }) {
  const { locale, t } = useI18n();
  const { snapshot, failed, received } = useSnapshot(url);
  const [selected, setSelected] = useState<number | null>(null);
  const checks = snapshot?.checks ?? [];
  const last = checks.at(-1);
  const first = checks[0];
  const offset = SLOTS - checks.length;

  const time = (at: number) => {
    const zone = new Intl.DateTimeFormat(locale, { timeZoneName: "short" })
      .formatToParts(at)
      .find((part) => part.type === "timeZoneName")?.value;
    return [CLOCK.format(at), zone].filter(Boolean).join(" ");
  };
  const stateOf = (check: Check) =>
    check.reachable ? t("status.reachable") : t("status.unreachable");
  const describe = (check: Check) =>
    check.reachable ? `${stateOf(check)} · ${check.latency_ms} ms` : stateOf(check);

  const top = ceiling(Math.max(0, ...checks.filter((c) => c.reachable).map((c) => c.latency_ms)));
  const failures = checks.filter((check) => !check.reachable).length;
  const summary = failed
    ? snapshot
      ? `${t("status.unavailable")} ${t("status.stale", { time: time(received) })}`
      : t("status.unavailable")
    : t("status.history", { count: checks.length, failed: failures });
  const state = failed || !last ? "unknown" : last.reachable ? "ok" : "down";
  const stateLabel = failed ? t("status.unknown") : last ? stateOf(last) : t("status.checking");
  const headline = failed || !last ? stateLabel : describe(last);
  const index = selected === null ? null : Math.min(selected, checks.length - 1);
  const focus = index === null ? undefined : checks[index];

  const onKeyDown = (event: React.KeyboardEvent) => {
    if (checks.length === 0) return;
    const current = index ?? checks.length - 1;
    const next = {
      ArrowLeft: Math.max(0, current - 1),
      ArrowRight: Math.min(checks.length - 1, current + 1),
      Home: 0,
      End: checks.length - 1,
    }[event.key];
    if (next === undefined) return;
    event.preventDefault();
    setSelected(next);
  };

  return (
    <section id="status">
      <h2>
        {t("status.title")}
        <Anchor id="status" />
      </h2>
      <figure className={failed && snapshot ? "chart stale" : "chart"}>
        <figcaption className={`now ${state}`}>
          <i aria-hidden="true" />
          {headline}
        </figcaption>
        <p className="sr-only" role="status">
          {stateLabel}
        </p>
        <div className="scale" aria-hidden="true">
          <span>{top} ms</span>
          <span>0</span>
        </div>
        <div
          className="plot"
          tabIndex={0}
          role="group"
          aria-label={`${t("status.keys")} ${summary}`}
          onKeyDown={onKeyDown}
          onFocus={() => setSelected((current) => current ?? checks.length - 1)}
          onBlur={() => setSelected(null)}
          onMouseLeave={() => setSelected(null)}
        >
          <svg viewBox={`0 0 ${SLOTS * SLOT} ${HEIGHT}`} preserveAspectRatio="none" aria-hidden="true">
            <defs>
              <pattern id="status-down" width="4" height="8" patternUnits="userSpaceOnUse">
                <rect className="hatch-base" width="4" height="8" />
                <rect className="hatch-line" width="4" height="3" />
              </pattern>
            </defs>
            {checks.map((check, at) => {
              const height = check.reachable
                ? Math.max(2, (Math.min(check.latency_ms, top) / top) * HEIGHT)
                : HEIGHT;
              const x = (offset + at) * SLOT;
              return (
                <g key={check.at} onMouseEnter={() => setSelected(at)}>
                  <rect className="hit" x={x} y={0} width={SLOT} height={HEIGHT} />
                  <rect
                    className={`${check.reachable ? "ok" : "down"}${index === at ? " on" : ""}`}
                    x={x + (SLOT - BAR) / 2}
                    y={HEIGHT - height}
                    width={BAR}
                    height={height}
                  />
                </g>
              );
            })}
          </svg>
          {focus && index !== null && (
            <div
              className={`tip${offset + index > SLOTS / 2 ? " left" : ""}`}
              role="status"
              style={{ left: `${((offset + index + 0.5) / SLOTS) * 100}%` }}
            >
              <strong>{time(focus.at)}</strong>
              <span>{describe(focus)}</span>
              {focus.status !== null && <span>HTTP {focus.status}</span>}
            </div>
          )}
        </div>
        <div className="times" aria-hidden="true">
          <span>{first && offset === 0 ? time(first.at) : ""}</span>
          <span>{last ? time(last.at) : ""}</span>
        </div>
        <div className="legend">
          <span className="key ok">
            <i aria-hidden="true" />
            {t("status.reachable")}
          </span>
          <span className="key down">
            <i aria-hidden="true" />
            {t("status.unreachable")}
          </span>
          <span className="summary">{summary}</span>
        </div>
      </figure>
    </section>
  );
}
