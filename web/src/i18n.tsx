import { createContext, useContext } from "react";

import en from "./locales/en.json";
import ko from "./locales/ko.json";
import type { Locale } from "./types";

export type MessageKey = keyof typeof en;

export const LOCALES: { code: Locale; label: string }[] = [
  { code: "en", label: "English" },
  { code: "ko", label: "한국어" },
];

const catalogs: Record<Locale, Record<string, string>> = { en, ko };

export interface I18n {
  locale: Locale;
  t: (key: MessageKey, values?: Record<string, string | number>) => string;
  translated: (key: string, source: string) => string;
}

export function createI18n(locale: Locale): I18n {
  const catalog = catalogs[locale];
  return {
    locale,
    t: (key, values = {}) =>
      Object.entries(values).reduce(
        (text, [name, value]) => text.replaceAll(`{{${name}}}`, String(value)),
        catalog[key] ?? key,
      ),
    translated: (key, source) => catalog[key] ?? source,
  };
}

export const I18nContext = createContext<I18n>(createI18n("en"));

export function useI18n(): I18n {
  return useContext(I18nContext);
}

export function Rich({ text }: { text: string }) {
  return text
    .split(/<code>(.*?)<\/code>/)
    .map((part, index) => (index % 2 === 1 ? <code key={index}>{part}</code> : part));
}
