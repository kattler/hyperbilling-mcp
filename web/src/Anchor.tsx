import { useI18n } from "./i18n";

export function Anchor({ id }: { id: string }) {
  const { t } = useI18n();
  return (
    <a className="anchor" href={`#${encodeURIComponent(id)}`} aria-label={t("ui.permalink")}>
      #
    </a>
  );
}
