// A rough stand-in for the core's search (crates/tasqlet-core/src/search.rs), only for the browser
// preview. The app always searches in Rust; this keeps the preview usable.
import type { Item } from "./types";

/** The words of a query, lower case; what the interface marks in the hits. */
export function searchTerms(query: string): string[] {
  return query
    .split(/\s+/)
    .map((w) => w.toLowerCase())
    .filter(Boolean);
}

function weight(item: Item, term: string): number {
  const has = (s: string | null | undefined) => (s ?? "").toLowerCase().includes(term);
  if (has(item.title)) return 100;
  if (has(item.delegated_to)) return 60;
  if (item.tags.some(has)) return 50;
  if (item.resources.some((r) => has(r.target) || has(r.shown) || has(r.label))) return 40;
  if (item.checklist.some((c) => has(c.text))) return 30;
  if (has(item.note)) return 20;
  return 0;
}

/** Items containing every word, best first: title before note, open before finished. */
export function mockSearch(items: Item[], query: string, limit = 50): Item[] {
  const terms = searchTerms(query)
    .map((w) => w.replace(/^@/, ""))
    .filter(Boolean);
  if (!terms.length) return [];
  const scored: { item: Item; score: number }[] = [];
  for (const item of items) {
    const weights = terms.map((w) => weight(item, w));
    if (weights.every((w) => w > 0)) scored.push({ item, score: weights.reduce((a, b) => a + b, 0) });
  }
  scored.sort(
    (a, b) =>
      b.score - a.score ||
      Number(a.item.state === "done") - Number(b.item.state === "done") ||
      b.item.updated_at.localeCompare(a.item.updated_at),
  );
  return scored.slice(0, limit).map((s) => s.item);
}

/** The finished items, latest first (the search without a query). */
export function mockRecentlyDone(items: Item[], limit = 10): Item[] {
  return items
    .filter((i) => i.state === "done")
    .sort((a, b) => (b.done_at ?? "").localeCompare(a.done_at ?? ""))
    .slice(0, limit);
}
