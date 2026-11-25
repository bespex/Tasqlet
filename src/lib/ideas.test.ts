import { describe, expect, it } from "vitest";
import { act, capture, loadSnapshot, mock, undo } from "./backend";
import { ageLabel, bullets } from "./format";
import { setLang } from "./i18n";

describe("brainstorming an idea", () => {
  it("a keyword grows over several thoughts into a task in focus, nothing is lost", async () => {
    mock.reset();
    await capture("Onboarding vereinfachen", "idea");
    const idea = () => mock.snapshot().items.find((i) => i.title === "Onboarding vereinfachen")!;
    const id = idea().id;
    await act({ kind: "brainstorm", id, text: "Willkommensmail kürzen" });
    await act({ kind: "brainstorm", id, text: "Checkliste für Tag 1" });
    await act({ kind: "brainstorm", id, text: "https://wiki.example.com/onboarding" });
    expect(idea().note).toBe("- Willkommensmail kürzen\n- Checkliste für Tag 1");
    expect(idea().resources.map((r) => r.target)).toEqual(["https://wiki.example.com/onboarding"]);
    await act({ kind: "send_to", id, to: "focus" });
    const now = idea();
    expect(now.state).toBe("focus");
    expect(bullets(now.note)).toHaveLength(2);
    expect(now.resources).toHaveLength(1);
    await expect(act({ kind: "brainstorm", id, text: "  " })).rejects.toThrow();
    await undo(); // back to the idea, thoughts kept
    expect(idea().state).toBe("idea");
    expect(bullets(idea().note)).toHaveLength(2);
    expect((await loadSnapshot()).items.filter((i) => i.state === "idea").length).toBeGreaterThan(0);
  });
});

describe("helpers", () => {
  it("takes bullets from a note without markers and blanks", () => {
    expect(bullets("- eins\n\n* zwei\n• drei\nvier")).toEqual(["eins", "zwei", "drei", "vier"]);
    expect(bullets("")).toEqual([]);
  });

  it("says roughly how long ago", () => {
    setLang("en");
    const now = new Date("2026-11-25T12:00:00Z");
    const ago = (ms: number) => new Date(now.getTime() - ms).toISOString();
    expect(ageLabel(ago(10_000), now)).toBe("just now");
    expect(ageLabel(ago(5 * 60_000), now)).toBe("5 min ago");
    expect(ageLabel(ago(3 * 3_600_000), now)).toBe("3 h ago");
    expect(ageLabel(ago(2 * 86_400_000), now)).toBe("2 d ago");
    setLang("de");
  });
});
