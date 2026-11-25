import { describe, expect, it } from "vitest";
import { comboFromEvent, keyLabel } from "./keys";
import { setLang } from "./i18n";

const ev = (
  code: string,
  key: string,
  mods: Partial<Record<"ctrlKey" | "altKey" | "shiftKey" | "metaKey", boolean>> = {},
) => ({
  code,
  key,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...mods,
});

describe("global key recording", () => {
  it("records modifiers in a fixed order and the physical key", () => {
    expect(comboFromEvent(ev("KeyT", "T", { shiftKey: true, ctrlKey: true, altKey: true }))).toBe(
      "Ctrl+Alt+Shift+KeyT",
    );
    expect(comboFromEvent(ev("Digit1", "!", { metaKey: true, shiftKey: true }))).toBe("Shift+Super+Digit1");
  });
  it("waits while only modifiers are held", () => {
    expect(comboFromEvent(ev("ControlLeft", "Control", { ctrlKey: true }))).toBeNull();
  });
  it("takes a plain key only if it is a function key", () => {
    expect(comboFromEvent(ev("KeyT", "t"))).toBeNull();
    expect(comboFromEvent(ev("KeyT", "T", { shiftKey: true }))).toBeNull();
    expect(comboFromEvent(ev("F9", "F9"))).toBe("F9");
  });
});

describe("key labels", () => {
  it("speaks the language", () => {
    setLang("de");
    expect(keyLabel("Ctrl+Alt+Shift+KeyT")).toBe("Strg+Alt+Umschalt+T");
    expect(keyLabel("Super+Space")).toBe("Super+Leertaste");
    setLang("en");
    expect(keyLabel("Ctrl+Digit1")).toBe("Ctrl+1");
    expect(keyLabel("Ctrl+F")).toBe("Ctrl+F");
  });
});
