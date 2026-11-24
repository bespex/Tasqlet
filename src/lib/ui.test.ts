import { afterEach, describe, expect, it, vi } from "vitest";
import { de } from "./i18n.de";
import { setLang, t } from "./i18n";
import { DEFAULT_THEME, PRESETS, applyTheme } from "./theme";
import { TOAST_MS, hideToast, showToast, toast } from "./toast.svelte";

/** Enough of an element for applyTheme. */
function fakeElement() {
  const attrs = new Map<string, string>();
  return {
    attrs,
    setAttribute: (k: string, v: string) => void attrs.set(k, v),
    removeAttribute: (k: string) => void attrs.delete(k),
  } as unknown as HTMLElement & { attrs: Map<string, string> };
}

describe("theme", () => {
  it("defaults to Leitstand (D19)", () => {
    expect(DEFAULT_THEME).toEqual({ name: "Leitstand", palette: "fjord", focus: "strong", chrome: "raised" });
  });

  it("sets palette and axes as attributes, default steps as none", () => {
    const el = fakeElement();
    applyTheme(DEFAULT_THEME, el);
    expect(Object.fromEntries(el.attrs)).toEqual({
      "data-tq": "fjord",
      "data-focus": "strong",
      "data-chrome": "raised",
    });
    applyTheme(
      PRESETS.find((p) => p.name === "Automatisch")!,
      el,
    );
    expect(el.attrs.size).toBe(0);
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
