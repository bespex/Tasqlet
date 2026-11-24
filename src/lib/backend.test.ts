import { describe, expect, it } from "vitest";
import { inTauri, mock, schemaVersion } from "./backend";

describe("backend outside Tauri", () => {
  it("is not running in Tauri", () => {
    expect(inTauri()).toBe(false);
  });

  it("falls back to the mock", async () => {
    expect(await schemaVersion()).toBe(mock.schemaVersion);
  });
});
