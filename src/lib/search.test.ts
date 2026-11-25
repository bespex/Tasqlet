import { describe, expect, it } from "vitest";
import { mock, search } from "./backend";
import { mockSearch, searchTerms } from "./mock-search";

describe("search in the preview", () => {
  it("splits a query into lower case words", () => {
    expect(searchTerms("  Webhook   CORE-418 ")).toEqual(["webhook", "core-418"]);
    expect(searchTerms("   ")).toEqual([]);
  });

  it("finds by title, note, step, resource and delegate, all words needed", async () => {
    mock.reset();
    const items = mock.snapshot().items;
    expect(mockSearch(items, "webhook").length).toBeGreaterThan(0);
    expect(mockSearch(items, "webhook gibtsnicht")).toEqual([]);
    expect(mockSearch(items, "   ")).toEqual([]);
    const person = items.find((i) => i.delegated_to)!;
    expect(mockSearch(items, `@${person.delegated_to!.toLowerCase()}`).map((i) => i.id)).toContain(person.id);
  });

  it("a title hit comes before a note hit", () => {
    const items = mock.snapshot().items;
    const hits = mockSearch(items, "core");
    const firstTitle = hits.findIndex((i) => i.title.toLowerCase().includes("core"));
    const firstOther = hits.findIndex((i) => !i.title.toLowerCase().includes("core"));
    if (firstOther >= 0) expect(firstTitle).toBeLessThan(firstOther);
  });

  it("without words it lists the finished ones, latest first", async () => {
    mock.reset();
    const result = await search("");
    expect(result.recent).toBe(true);
    expect(result.items.every((i) => i.state === "done")).toBe(true);
    const days = result.items.map((i) => i.done_at ?? "");
    expect([...days].sort().reverse()).toEqual(days);
  });
});
