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

describe("the intermezzo bar's actions", () => {
  const inter = () => mock.snapshot().items.filter((i) => i.state === "intermezzo");

  it("taking a pill over makes it #1 and the old one #2; undo puts it back", async () => {
    mock.reset();
    const pill = inter()[1];
    const before = mock.snapshot().items.find((i) => i.active)!;
    await act({ kind: "start", id: pill.id });
    const after = mock
      .snapshot()
      .items.filter((i) => i.state === "focus")
      .sort((a, b) => a.order - b.order);
    expect(after[0].id).toBe(pill.id);
    expect(after[0].active).toBe(true);
    expect(after[1].id).toBe(before.id);
    expect(after[1].last_active).toBe(true);
    expect(inter().map((i) => i.order)).toEqual(inter().map((_, k) => k));
    await undo();
    expect(inter().some((i) => i.id === pill.id)).toBe(true);
  });

  it("ticking a pill off finishes it and closes the gap", async () => {
    mock.reset();
    const [first, second] = inter();
    await act({ kind: "finish", id: first.id });
    expect(inter().map((i) => i.id)).not.toContain(first.id);
    expect(inter()[0].id).toBe(second.id);
    expect(inter()[0].order).toBe(0);
  });
});

describe("settings in the preview", () => {
  it("keeps language, position, autostart and forgets a Jira project", async () => {
    const { getSettings, setSettings, setAutostart, forgetJira } = await import("./backend");
    const first = await getSettings();
    expect(first.jira.length).toBeGreaterThan(0);
    await setSettings("en", "center");
    await setAutostart(true);
    await forgetJira(first.jira[0].project);
    const now = await getSettings();
    expect([now.language, now.popover_position, now.autostart]).toEqual(["en", "center", true]);
    expect(now.jira.map((j) => j.project)).not.toContain(first.jira[0].project);
  });
});

describe("elaboration helpers", () => {
  it("understands a typed day", async () => {
    const { parseDay } = await import("./format");
    const today = new Date(2026, 10, 25);
    expect(parseDay("", today)).toBeNull();
    expect(parseDay("2026-11-27", today)).toBe("2026-11-27");
    expect(parseDay("27.11.", today)).toBe("2026-11-27");
    expect(parseDay("1.2.27", today)).toBe("2027-02-01");
    expect(parseDay("3.4.2028", today)).toBe("2028-04-03");
    expect(parseDay("morgen", today)).toBe("2026-11-26");
    expect(parseDay("Today", today)).toBe("2026-11-25");
    expect(parseDay("31.02.", today)).toBeUndefined();
    expect(parseDay("bald", today)).toBeUndefined();
  });

  it("edits date and steps of any item", async () => {
    mock.reset();
    const item = mock.snapshot().items.find((i) => i.checklist.length > 1)!;
    await act({ kind: "set_date", id: item.id, date: "2026-12-01" });
    expect(mock.snapshot().items.find((i) => i.id === item.id)!.date).toBe("2026-12-01");
    await act({ kind: "set_date", id: item.id, date: null });
    expect(mock.snapshot().items.find((i) => i.id === item.id)!.date).toBeNull();
    const step = item.checklist[0];
    await act({ kind: "rename_check", id: item.id, check: step.id, text: "  neu  " });
    expect(mock.snapshot().items.find((i) => i.id === item.id)!.checklist[0].text).toBe("neu");
    await expect(act({ kind: "rename_check", id: item.id, check: step.id, text: " " })).rejects.toThrow();
    await act({ kind: "remove_check", id: item.id, check: step.id });
    expect(mock.snapshot().items.find((i) => i.id === item.id)!.checklist).toHaveLength(item.checklist.length - 1);
  });
});
