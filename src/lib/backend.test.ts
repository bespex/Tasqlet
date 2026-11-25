import { describe, expect, it } from "vitest";
import { act, inTauri, loadSnapshot, mock, schemaVersion, undo, watchChanged } from "./backend";
import { mockSnapshot } from "./mock-data";

describe("backend outside Tauri", () => {
  it("is not running in Tauri", () => {
    expect(inTauri()).toBe(false);
  });

  it("falls back to the mock", async () => {
    expect(await schemaVersion()).toBe(mock.schemaVersion);
    expect((await loadSnapshot()).items.length).toBeGreaterThan(0);
  });

  it("tells listeners in the preview about changes until they stop", async () => {
    mock.reset();
    let calls = 0;
    const stop = await watchChanged(() => calls++);
    await act({ kind: "pause" });
    expect(calls).toBe(1);
    stop();
    await act({ kind: "resume" });
    expect(calls).toBe(1);
  });
});

describe("mock data", () => {
  const items = mockSnapshot(new Date(2026, 10, 24, 10, 0)).items;
  const focus = items.filter((i) => i.state === "focus");

  it("has exactly one active item, on #1 of the focus list", () => {
    const active = items.filter((i) => i.active);
    expect(active).toHaveLength(1);
    expect(active[0].state).toBe("focus");
    expect(active[0].order).toBe(0);
  });

  it("keeps the focus order gapless", () => {
    expect(focus.map((i) => i.order).sort()).toEqual(focus.map((_, k) => k));
  });

  it("computes dates relative to now", () => {
    expect(items[0].date).toBe("2026-11-24");
  });
});

describe("act in the preview", () => {
  const focusTitles = async () =>
    (await loadSnapshot()).items
      .filter((i) => i.state === "focus")
      .sort((a, b) => a.order - b.order)
      .map((i) => `${i.title.slice(0, 4)}${i.active ? "*" : ""}`);

  it("finishing the active task leaves the next one ready; undo brings it back", async () => {
    mock.reset();
    const first = (await loadSnapshot()).items.find((i) => i.active)!;
    await act({ kind: "finish", id: first.id });
    const after = await loadSnapshot();
    expect(after.items.some((i) => i.active)).toBe(false);
    expect(after.items.find((i) => i.id === first.id)!.state).toBe("done");
    await undo();
    expect((await focusTitles())[0]).toMatch(/\*$/);
  });

  it("pause and resume toggle the break", async () => {
    mock.reset();
    await act({ kind: "pause" });
    expect((await loadSnapshot()).on_break).toBe(true);
    await act({ kind: "resume" });
    expect((await loadSnapshot()).on_break).toBe(false);
  });

  it("checks steps, saves the note without an undo step and refuses blank steps", async () => {
    mock.reset();
    const item = (await loadSnapshot()).items.find((i) => i.active)!;
    await act({ kind: "add_check", id: item.id, text: " write test " });
    await expect(act({ kind: "add_check", id: item.id, text: "  " })).rejects.toThrow();
    const step = (await loadSnapshot()).items.find((i) => i.id === item.id)!.checklist.at(-1)!;
    expect(step.text).toBe("write test");
    await act({ kind: "toggle_check", id: item.id, check: step.id });
    await act({ kind: "set_note", id: item.id, note: "typed" });
    await undo(); // undoes the tick, not the note
    const now = (await loadSnapshot()).items.find((i) => i.id === item.id)!;
    expect(now.checklist.at(-1)!.done).toBe(false);
  });

  it("sending the active task back puts the next one on ready", async () => {
    mock.reset();
    const first = (await loadSnapshot()).items.find((i) => i.active)!;
    await act({ kind: "to_back", id: first.id });
    const snap = await loadSnapshot();
    expect(snap.items.some((i) => i.active)).toBe(false);
    const top = snap.items.filter((i) => i.state === "focus").sort((a, b) => a.order - b.order)[0];
    expect(top.id).not.toBe(first.id);
  });
});
