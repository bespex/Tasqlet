// Themes: a palette and two axes (docs/KONZEPT.md 7), applied as attributes on <html>; the CSS in
// app.css maps them to the role tokens. Plan 1.5 ships Automatisch, Papier, Graphit and Fjord with
// the steps the presets below need; all palettes, steps, theme files and settings follow in 2.6.

export type Palette = "auto" | "papier" | "graphit" | "fjord";
export type FocusStep = "calm" | "strong";
export type ChromeStep = "flat" | "raised";

export interface Theme {
  name: string;
  palette: Palette;
  focus: FocusStep;
  chrome: ChromeStep;
}

/** Presets available so far. "Leitstand" is the preferred style and the default (D19). */
export const PRESETS: Theme[] = [
  { name: "Leitstand", palette: "fjord", focus: "strong", chrome: "raised" },
  { name: "Automatisch", palette: "auto", focus: "calm", chrome: "flat" },
  { name: "Papier", palette: "papier", focus: "calm", chrome: "flat" },
  { name: "Graphit", palette: "graphit", focus: "calm", chrome: "flat" },
];

export const DEFAULT_THEME: Theme = PRESETS[0];

/** Put the theme on an element (by default <html>). Default steps leave no attribute. */
export function applyTheme(theme: Theme, el: HTMLElement = document.documentElement): void {
  const set = (name: string, value: string, fallback: string) => {
    if (value === fallback) el.removeAttribute(name);
    else el.setAttribute(name, value);
  };
  set("data-tq", theme.palette, "auto");
  set("data-focus", theme.focus, "calm");
  set("data-chrome", theme.chrome, "flat");
}
