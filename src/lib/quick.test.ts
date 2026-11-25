import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { capture, loadSnapshot, mock, parseQuick, undo } from "./backend";
import { dateLabel } from "./format";
import { setLang } from "./i18n";

const focusTitles = async () =>
  (await loadSnapshot()).items
    .filter((i) => i.state === "focus")
    .sort((a, b) => a.order - b.order)
    .map((i) => (i.active ? `*${i.title}` : i.title));

describe("quick entry in the preview", () => {
  beforeEach(() => mock.reset());

  it("recognises person, date and links", async () => {
    const e = await parseQuick("Review https://github.com/example/core/pull/42 !morgen @Jonas");
    expect(e.title).toBe("Review");
    expect(e.delegate_to).toBe("Jonas");
    expect(e.date).not.toBeNull();
    expect(e.resources.map((r) => r.shown)).toEqual(["PR #42"]);
  });

  it("turns keys of a known Jira project into resources and keeps them in the title", async () => {
    const e = await parseQuick("CORE-422 klären");
    expect(e.title).toBe("CORE-422 klären");
    expect(e.resources[0].target).toBe("https://jira.example.com/browse/CORE-422");
  });

  it("places by key: now takes over, focus goes to the end, undo goes back", async () => {
    const before = await focusTitles();
    const saved = await capture("Hotfix ausrollen", "now");
    expect(saved).toMatchObject({ placed: "now", position: 1, replaced: true });
    const after = await focusTitles();
    expect(after[0]).toBe("*Hotfix ausrollen");
    expect(after[1]).toBe(before[0].slice(1));

    const end = await capture("Später lesen", "focus");
    expect(end.position).toBe(after.length + 1);

    await undo();
    await undo();
    expect(await focusTitles()).toEqual(before);
    await expect(undo()).rejects.toThrow();
  });

  it("a person sends it to Delegated whatever the key", async () => {
    const saved = await capture("Load-Test @Jonas", "now");
    expect(saved).toMatchObject({ placed: "delegated", delegated_to: "Jonas" });
  });

  it("refuses an empty entry", async () => {
    await expect(capture("   ", "idea")).rejects.toThrow();
  });
});

describe("date label", () => {
  afterEach(() => setLang("en"));
  const today = new Date(2026, 10, 24); // Tuesday

  it("names near days and shows the rest with weekday", () => {
    setLang("de");
    expect(dateLabel("2026-11-24", today)).toBe("Heute");
    expect(dateLabel("2026-11-25", today)).toBe("Morgen");
    expect(dateLabel("2026-11-23", today)).toBe("Gestern");
    expect(dateLabel("2026-11-27", today)).toBe("Fr, 27.11.");
    setLang("en");
    expect(dateLabel("2026-11-27", today)).toBe("Fri, 27.11.");
  });
});
