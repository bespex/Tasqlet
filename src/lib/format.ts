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

/**
 * A day typed by hand: `2026-11-27`, `27.11.`, `27.11.26`, `27.11.2026`, "today", "tomorrow" (also in
 * German). Empty text clears the date (null); anything else yields `undefined` (not understood).
 * The year defaults to this year; a day that does not exist (31.02.) is not understood.
 */
export function parseDay(text: string, today: Date = new Date()): string | null | undefined {
  const v = text.trim().toLowerCase();
  if (!v) return null;
  const pad = (n: number) => String(n).padStart(2, "0");
  const iso = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  const base = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  if (v === "today" || v === "heute") return iso(base);
  if (v === "tomorrow" || v === "morgen") return iso(new Date(base.getFullYear(), base.getMonth(), base.getDate() + 1));
  let y: number, m: number, d: number;
  let hit = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(v);
  if (hit) [y, m, d] = [+hit[1], +hit[2], +hit[3]];
  else if ((hit = /^(\d{1,2})\.(\d{1,2})\.(\d{2}|\d{4})?$/.exec(v))) {
    [m, d] = [+hit[2], +hit[1]];
    y = hit[3] ? (hit[3].length === 2 ? 2000 + +hit[3] : +hit[3]) : base.getFullYear();
  } else return undefined;
  const date = new Date(y, m - 1, d);
  return date.getFullYear() === y && date.getMonth() === m - 1 && date.getDate() === d ? iso(date) : undefined;
}
