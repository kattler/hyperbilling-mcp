import { useEffect, useState } from "react";

import { useI18n } from "./i18n";

type Theme = "dark" | "light";

const KEY = "hyperbilling-mcp-theme";
const LIGHT = "(prefers-color-scheme: light)";

function stored(): Theme | null {
  try {
    const value = window.localStorage.getItem(KEY);
    return value === "dark" || value === "light" ? value : null;
  } catch {
    return null;
  }
}

function system(): Theme {
  return window.matchMedia(LIGHT).matches ? "light" : "dark";
}

// Called before the first render, so a stored choice paints with the page.
export function applyStoredTheme() {
  const theme = stored();
  if (theme) document.documentElement.dataset.theme = theme;
}

export function ThemeToggle() {
  const { t } = useI18n();
  const [theme, setTheme] = useState<Theme>(() => stored() ?? system());

  useEffect(() => {
    const media = window.matchMedia(LIGHT);
    const follow = () => {
      if (!stored()) setTheme(system());
    };
    media.addEventListener("change", follow);
    return () => media.removeEventListener("change", follow);
  }, []);

  const toggle = () => {
    const next = theme === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = next;
    try {
      window.localStorage.setItem(KEY, next);
    } catch {
      // The choice still holds for this page view.
    }
    setTheme(next);
  };

  return (
    <button
      className="theme"
      type="button"
      aria-label={theme === "dark" ? t("ui.theme.light") : t("ui.theme.dark")}
      onClick={toggle}
    >
      <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
        {theme === "dark" ? (
          <>
            <circle cx="8" cy="8" r="3" />
            <path d="M8 1.5v1.5M8 13v1.5M1.5 8H3M13 8h1.5M3.4 3.4l1.1 1.1M11.5 11.5l1.1 1.1M3.4 12.6l1.1-1.1M11.5 4.5l1.1-1.1" />
          </>
        ) : (
          <path d="M13.5 9.5A5.5 5.5 0 0 1 6.5 2.5a5.5 5.5 0 1 0 7 7z" />
        )}
      </svg>
    </button>
  );
}
