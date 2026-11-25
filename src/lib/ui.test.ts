import { afterEach, describe, expect, it, vi } from "vitest";
import { de } from "./i18n.de";
import { setLang, t } from "./i18n";
import { DEFAULT_APPEARANCE, appearanceOf, applyAppearance } from "./theme";
import { TOAST_MS, hideToast, showToast, toast } from "./toast.svelte";

/** Enough of an element for applyAppearance. */
function fakeElement() {
  const attrs = new Map<string, string>();
  return {
    attrs,
    setAttribute: (k: string, v: string) => void attrs.set(k, v),
    removeAttribute: (k: string) => void attrs.delete(k),
  } as unknown as HTMLElement & { attrs: Map<string, string> };
}

describe("appearance (plan 2.6)", () => {
  it("defaults to the former Leitstand: Fjord, task emphasised, bars set off (D19)", () => {
    expect(DEFAULT_APPEARANCE.theme).toBe("fjord");
    expect(DEFAULT_APPEARANCE.style).toMatchObject({ chrome: "raised", emphasis: "strong" });
    expect(appearanceOf(undefined)).toEqual(DEFAULT_APPEARANCE);
  });

  it("makes stored values safe: unknown themes and steps fall back, a full preset is stored by name", () => {
    expect(appearanceOf({ theme: "nope", style: "calm" })).toEqual({ theme: "auto", style: "calm" });
    expect(appearanceOf({ theme: "sage", style: { chrome: "wild" } })).toEqual({ theme: "sage", style: "calm" });
  });

  it("puts theme and all seven axes on the element as attributes", () => {
    const el = fakeElement();
    applyAppearance({ theme: "paper", style: "stage" }, el);
    expect(Object.fromEntries(el.attrs)).toEqual({
      "data-sx-theme": "paper",
      "data-sx-shape": "soft",
      "data-sx-structure": "surfaces",
      "data-sx-chrome": "contrast",
      "data-sx-emphasis": "stage",
      "data-sx-density": "airy",
      "data-sx-color": "colorful",
      "data-sx-effects": "rich",
    });
  });
});

describe("i18n", () => {
  afterEach(() => setLang("en"));

  it("uses the English text as key and falls back to it", () => {
    setLang("de");
    expect(t("Focus")).toBe("Fokus");
    expect(t("Not translated yet")).toBe("Not translated yet");
    setLang("en");
    expect(t("Focus")).toBe("Focus");
  });

  it("fills in values", () => {
    setLang("de");
    expect(t("Restoring failed: {error}", { error: "kaputt" })).toBe("Wiederherstellen fehlgeschlagen: kaputt");
    expect(t("{n} left", { n: 3 })).toBe("3 left");
  });

  it("keeps the placeholders of every translation", () => {
    for (const [en, german] of Object.entries(de)) {
      const names = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
      expect(names(german), en).toEqual(names(en));
    }
  });
});

describe("toast", () => {
  afterEach(() => {
    hideToast();
    vi.useRealTimers();
  });

  it("shows one message and hides it after a while, longer with undo", () => {
    vi.useFakeTimers();
    showToast({ text: "Als Idee gespeichert" });
    expect(toast.current?.text).toBe("Als Idee gespeichert");
    vi.advanceTimersByTime(TOAST_MS.plain + 1);
    expect(toast.current).toBeNull();

    showToast({ text: "Erledigt", undo: () => {} });
    vi.advanceTimersByTime(TOAST_MS.plain + 1);
    expect(toast.current?.text).toBe("Erledigt");
    vi.advanceTimersByTime(TOAST_MS.undo);
    expect(toast.current).toBeNull();
  });

  it("a new message replaces the old one", () => {
    showToast({ text: "eins" });
    showToast({ text: "zwei" });
    expect(toast.current?.text).toBe("zwei");
  });
});
