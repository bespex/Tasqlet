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

/** How long ago, roughly and unobtrusive: "just now", "5 min ago", "3 h ago", "2 d ago". */
export function ageLabel(iso: string, now: Date = new Date()): string {
  const minutes = Math.max(0, Math.floor((now.getTime() - new Date(iso).getTime()) / 60_000));
  if (minutes < 1) return t("just now");
  if (minutes < 60) return t("{n} min ago", { n: minutes });
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return t("{n} h ago", { n: hours });
  return t("{n} d ago", { n: Math.floor(hours / 24) });
}

/** The bullets of a note: its non-blank lines without a leading list marker. */
export function bullets(note: string): string[] {
  return note
    .split("\n")
    .map((l) => l.trim().replace(/^[-*•]\s*/, ""))
    .filter(Boolean);
}
