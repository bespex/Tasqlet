// The interface in English or German, as in CodeDeck: texts stay in English in the code, where they
// are read; the English text is also the key into the German dictionary (`i18n.de.ts`), so a text
// not translated yet simply shows in English.
//
//   t("Focus")                          plain text
//   t("{n} items in focus", { n: 3 })   text with values

import { de } from "./i18n.de";

export type Lang = "en" | "de";

/** The system language: German if it starts with "de", else English. */
export function systemLang(): Lang {
  return typeof navigator !== "undefined" && navigator.language.toLowerCase().startsWith("de") ? "de" : "en";
}

let lang: Lang = systemLang();

/** Switch the language (tests, later the settings of plan 2.6). */
export function setLang(next: Lang): void {
  lang = next;
}

export function currentLang(): Lang {
  return lang;
}

/** The text in the current language, with `{name}` replaced by `values.name`. */
export function t(text: string, values?: Record<string, string | number>): string {
  const translated = lang === "de" ? (de[text] ?? text) : text;
  if (!values) return translated;
  return translated.replace(/\{(\w+)\}/g, (all, name: string) => (name in values ? String(values[name]) : all));
}
