import { describe, expect, it } from "vitest";
import { dropIndex, moveDelta } from "./queue";

describe("queue drag", () => {
  const mids = [10, 30, 50, 70];

  it("counts the rows above the pointer", () => {
    expect(dropIndex(0, mids)).toBe(0);
    expect(dropIndex(35, mids)).toBe(2);
    expect(dropIndex(999, mids)).toBe(4);
  });

  it("turns a drop slot into places to move", () => {
    expect(moveDelta(1, 3)).toBe(1); // row 1 goes behind row 2
    expect(moveDelta(1, 4)).toBe(2); // to the end
    expect(moveDelta(3, 1)).toBe(-2);
    expect(moveDelta(2, 2)).toBe(0); // dropped on its own place
    expect(moveDelta(2, 3)).toBe(0); // …or right behind itself
  });
});
