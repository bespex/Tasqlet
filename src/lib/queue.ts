// Pure helpers of the queue (docs/KONZEPT.md 6.1, 6.6), kept apart from the component so they can
// be tested without a DOM.

/** From this many items in focus the interface asks "Is that all urgent?" (docs/KONZEPT.md 4). */
export const MANY_IN_FOCUS = 8;

/** Where a dragged row would land: the number of rows whose middle lies above `y`. */
export function dropIndex(y: number, middles: number[]): number {
  return middles.filter((m) => m < y).length;
}

/**
 * Places to move for "Move" when the row at `from` is dropped before the row at `drop` (an index into
 * the list as it is, `drop` may be one past the end). Dropping next to itself gives 0.
 */
export function moveDelta(from: number, drop: number): number {
  const to = drop > from ? drop - 1 : drop;
  return to - from;
}
