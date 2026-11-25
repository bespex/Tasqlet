// Small formatting helpers for the interface. Dates are only shown, never judged (D2): no colours,
// no "overdue", just the day.
import { currentLang, t } from "./i18n";

/** "Heute", "Morgen", "Gestern", else "Fr, 27.11." (weekday short, day and month). */
export function dateLabel(isoDay: string, today: Date = new Date()): string {
  const [y, m, d] = isoDay.split("-").map(Number);
  const date = new Date(y, m - 1, d);
  const start = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  const days = Math.round((date.getTime() - start.getTime()) / 86_400_000);
  if (days === 0) return t("Today");
  if (days === 1) return t("Tomorrow");
  if (days === -1) return t("Yesterday");
  const locale = currentLang() === "de" ? "de-DE" : "en-GB";
  const weekday = date.toLocaleDateString(locale, { weekday: "short" }).replace(/\.$/, "");
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${weekday}, ${pad(d)}.${pad(m)}.`;
}
