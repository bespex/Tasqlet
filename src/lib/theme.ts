// Appearance (plan 2.6): colour theme and style from @bespex/themes, the same choice as in every be spex
// app. The kit owns themes, styles, the check and the picker; Tasqlet chooses its default, the axes it
// offers, stores the choice in settings.json and puts it on <html>.

import { apply, normalize, type Appearance, type Axis } from "@bespex/themes";

/** The default (D19, before: preset "Leitstand"): Fjord, the current task stands out, bars set off. */
export const DEFAULT_APPEARANCE: Appearance = normalize({ theme: "fjord", style: { chrome: "raised" } });

/** The style axes that change something in Tasqlet; the picker offers only these. Density, colour
 *  of areas and effects have no place in the popover yet. */
export const AXES_USED: Axis[] = ["shape", "structure", "chrome", "emphasis"];

/** What the settings hold, made safe; nothing stored yet means the default. */
export function appearanceOf(stored: unknown): Appearance {
  return stored === undefined || stored === null ? DEFAULT_APPEARANCE : normalize(stored);
}

/** Put colour theme and style on an element (by default <html>). */
export function applyAppearance(value: Appearance, el?: HTMLElement): void {
  apply(value, el);
}
