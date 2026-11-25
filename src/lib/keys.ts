// Key combinations: as the global key is stored (`Ctrl+Alt+KeyT`, the names of KeyboardEvent.code,
// which the shell's parser reads) and as they are shown (`Strg+Alt+T`).

import { t } from "./i18n";

const MODIFIERS = ["Control", "Shift", "Alt", "Meta", "AltGraph", "OS", "Super", "Hyper"];

/** The combination a key press makes, or null while only modifiers are held. Without Ctrl, Alt or
 *  Super only F1 … F24 count: a plain letter as a global key would swallow it everywhere. */
export function comboFromEvent(e: Pick<KeyboardEvent, "key" | "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">) {
  if (MODIFIERS.includes(e.key) || !e.code) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(
    (m): m is string => !!m,
  );
  const fkey = /^F\d{1,2}$/.test(e.code);
  if (!fkey && !(e.ctrlKey || e.altKey || e.metaKey)) return null;
  return [...mods, e.code].join("+");
}

const ARROWS: Record<string, string> = { ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→" };

/** A combination for the eye, in the current language: `Ctrl+Alt+KeyT` → `Strg+Alt+T`. */
export function keyLabel(combo: string): string {
  return combo
    .split("+")
    .map((part) => {
      if (part === "Ctrl") return t("Ctrl");
      if (part === "Shift") return t("Shift");
      if (part === "Space") return t("Space");
      if (part in ARROWS) return ARROWS[part];
      const m = /^(?:Key|Digit|Numpad)(.+)$/.exec(part);
      return m ? m[1] : part;
    })
    .join("+");
}

/** A button's tooltip with its key: `Pause (Strg+P)`. */
export const withKey = (text: string, combo: string) => `${text} (${keyLabel(combo)})`;
