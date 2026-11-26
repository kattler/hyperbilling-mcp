import { useEffect, useRef, useState } from "react";

import { useI18n } from "./i18n";

function legacyCopy(text: string): Promise<void> {
  const area = document.createElement("textarea");
  area.value = text;
  area.readOnly = true;
  area.style.position = "fixed";
  area.style.opacity = "0";
  document.body.append(area);
  area.select();
  const copied = document.execCommand("copy");
  area.remove();
  return copied ? Promise.resolve() : Promise.reject(new Error("copy failed"));
}

// navigator.clipboard exists only on HTTPS and localhost.
function copy(text: string): Promise<void> {
  if (!navigator.clipboard) return legacyCopy(text);
  return navigator.clipboard.writeText(text).catch(() => legacyCopy(text));
}

export function CopyButton({ text }: { text: string }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const onCopy = () => {
    copy(text).then(
      () => {
        setCopied(true);
        window.clearTimeout(timer.current);
        timer.current = window.setTimeout(() => setCopied(false), 1600);
      },
      () => setCopied(false),
    );
  };

  return (
    <button
      className={copied ? "copy done" : "copy"}
      type="button"
      aria-label={copied ? t("ui.copied") : t("ui.copy")}
      aria-live="polite"
      onClick={onCopy}
    >
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
        {copied ? (
          <path d="M3 8.5l3.2 3.2L13 5" />
        ) : (
          <>
            <rect x="5.5" y="5.5" width="8" height="8" rx="1.5" />
            <path d="M10.5 5.5v-1.5a1.5 1.5 0 0 0-1.5-1.5h-5a1.5 1.5 0 0 0-1.5 1.5v5a1.5 1.5 0 0 0 1.5 1.5h1.5" />
          </>
        )}
      </svg>
    </button>
  );
}

export function Value({ text }: { text: string }) {
  return (
    <span className="copyable">
      <span className="name">{text}</span>
      <CopyButton text={text} />
    </span>
  );
}

export function Snippet({ text }: { text: string }) {
  return (
    <div className="snippet">
      <pre>{text}</pre>
      <CopyButton text={text} />
    </div>
  );
}
