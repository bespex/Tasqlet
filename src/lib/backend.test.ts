import { describe, expect, it } from "vitest";
import { inTauri, loadSnapshot, mock, schemaVersion, watchChanged } from "./backend";
import { mockSnapshot } from "./mock-data";

describe("backend outside Tauri", () => {
  it("is not running in Tauri", () => {
    expect(inTauri()).toBe(false);
  });

  it("falls back to the mock", async () => {
    expect(await schemaVersion()).toBe(mock.schemaVersion);
    expect((await loadSnapshot()).items.length).toBeGreaterThan(0);
  });

  it("does not listen for changes in the browser", async () => {
    const stop = await watchChanged(() => {
      throw new Error("never called");
    });
    expect(typeof stop).toBe("function");
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
