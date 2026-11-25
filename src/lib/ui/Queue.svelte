<script lang="ts">
  // The queue #2 … n (docs/KONZEPT.md 6.1, 6.6): one line per task. Keys: ↑/↓ select (↑ on the first
  // line back to the quick entry), Enter takes over, Alt+↑/↓ move, Alt+Home next. Sorting by pointer
  // goes through the grip (no HTML5 drag and drop, which WebKitGTK handles badly).
  import { tick } from "svelte";
  import { Link, ListStart, Maximize2, Play } from "@lucide/svelte";
  import { t } from "../i18n";
  import { withKey } from "../keys";
  import { dropIndex, moveDelta } from "../queue";
  import type { Act, Item } from "../types";
  import Chip from "./Chip.svelte";
  import Row from "./Row.svelte";

  let {
    items,
    run,
    open,
    onup,
    onelaborate,
  }: {
    items: Item[];
    /** Opens one resource of a task (ranks it first). */
    open: (id: string, resource: string) => void;
    run: (change: Act, message?: string) => Promise<boolean>;
    /** ↑ on the first line: back into the quick entry. */
    onup?: () => void;
    /** Opens the elaboration of a task. */
    onelaborate: (id: string) => void;
  } = $props();

  let list: HTMLElement;
  /** The task whose resources are unfolded below its line (the 🔗 on the line). */
  let unfolded = $state<string | null>(null);
  let selected = $state<string | null>(null);
  let drag = $state<{ id: string; from: number; drop: number; y: number } | null>(null);
  /** The copy of the dragged line that follows the pointer: position, size and what it shows. */
  let ghost = $state<{ x: number; y: number; w: number; h: number; num: number; title: string } | null>(null);

  const rows = () => [...list.querySelectorAll<HTMLElement>("[role=option]")];

  /** Focus the first line (from the quick entry's ↓). */
  export function enter() {
    rows()[0]?.focus();
  }

  /** The task a paste goes to: the line under the pointer, else the selected one if it has the focus. */
  export function target(): string | null {
    const hovered = list.querySelector<HTMLElement>("[role=option]:hover")?.dataset.id;
    if (hovered) return hovered;
    return selected && list.contains(document.activeElement) ? selected : null;
  }

  async function select(id: string) {
    selected = id;
    await tick();
    list.querySelector<HTMLElement>(`[data-id="${id}"]`)?.focus();
  }

  async function start(item: Item) {
    if (await run({ kind: "start", id: item.id })) {
      selected = null;
      onup?.();
    }
  }

  async function move(item: Item, change: Act) {
    if (await run(change)) await select(item.id);
  }

  function onkeydown(e: KeyboardEvent) {
    const i = items.findIndex((it) => it.id === selected);
    if (i < 0) return;
    const item = items[i];
    if (e.altKey && e.key === "ArrowUp") void move(item, { kind: "move_by", id: item.id, delta: -1 });
    else if (e.altKey && e.key === "ArrowDown") void move(item, { kind: "move_by", id: item.id, delta: 1 });
    else if (e.altKey && e.key === "Home") void move(item, { kind: "to_next", id: item.id });
    else if (e.key === "ArrowDown") void select(items[Math.min(i + 1, items.length - 1)].id);
    else if (e.key === "ArrowUp") {
      if (i === 0) onup?.();
      else void select(items[i - 1].id);
    } else if (e.key === "Home") void select(items[0].id);
    else if (e.key === "End") void select(items[items.length - 1].id);
    else if (e.key === "Enter") void start(item);
    else return;
    e.preventDefault();
  }

  function startDrag(e: PointerEvent, id: string) {
    if (e.button !== 0) return;
    e.preventDefault();
    const grip = e.currentTarget as HTMLElement;
    grip.setPointerCapture(e.pointerId);
    const from = items.findIndex((it) => it.id === id);
    drag = { id, from, drop: from, y: 0 };
    const box = grip.closest<HTMLElement>("[role=option]")?.getBoundingClientRect();
    // The line stays under the pointer where it was grabbed, only moving up and down.
    const grab = box ? e.clientY - box.top : 0;
    ghost = box
      ? { x: box.left, y: box.top, w: box.width, h: box.height, num: from + 2, title: items[from].title }
      : null;
    const middles = () =>
      rows().map((r) => {
        const b = r.getBoundingClientRect();
        return b.top + b.height / 2;
      });
    const over = (ev: PointerEvent) => {
      if (drag) {
        drag.drop = dropIndex(ev.clientY, middles());
        drag.y = slotY(drag.drop);
      }
      if (ghost) ghost.y = ev.clientY - grab;
    };
    const end = async (ev: PointerEvent) => {
      window.removeEventListener("pointermove", over);
      window.removeEventListener("pointerup", end);
      window.removeEventListener("pointercancel", end);
      over(ev);
      const done = drag;
      drag = null;
      ghost = null;
      const delta = done && ev.type === "pointerup" ? moveDelta(done.from, done.drop) : 0;
      if (delta) await move(items[from], { kind: "move_by", id, delta });
    };
    window.addEventListener("pointermove", over);
    window.addEventListener("pointerup", end);
    window.addEventListener("pointercancel", end);
  }

  /** Height of the gap between rows `drop - 1` and `drop` within the list, where the mark goes. */
  function slotY(drop: number): number {
    const all = rows();
    const top = list.getBoundingClientRect().top;
    if (all.length === 0) return 0;
    return drop < all.length
      ? all[drop].getBoundingClientRect().top - top
      : all[all.length - 1].getBoundingClientRect().bottom - top;
  }

  /** The drop mark is only shown if the drop would actually move something. */
  const marked = $derived(drag && moveDelta(drag.from, drag.drop) !== 0 ? drag.y : null);
</script>

<div class="list" role="listbox" tabindex="-1" aria-label={t("Focus")} bind:this={list} {onkeydown}>
  {#each items as item, i (item.id)}
    <Row
      id={item.id}
      num={i + 2}
      title={item.title}
      selected={item.id === selected}
      dragging={drag?.id === item.id}
      onselect={() => (selected = item.id)}
      ongrip={(e) => startDrag(e, item.id)}
    >
      {#snippet indicators()}
        {#if item.last_active}<span class="badge">{t("↩ last worked on")}</span>{/if}
        {#if item.resources.length}
          <button
            class="links"
            class:on={unfolded === item.id}
            title={t("Show resources")}
            aria-label={t("Show resources")}
            aria-expanded={unfolded === item.id}
            onclick={(e) => (e.stopPropagation(), (unfolded = unfolded === item.id ? null : item.id))}
          >
            <Link size={13} />{item.resources.length}
          </button>
        {/if}
      {/snippet}
      {#snippet tools()}
        <button
          class="ib"
          title={withKey(t("Elaborate"), "Ctrl+E")}
          aria-label={t("Elaborate")}
          onclick={(e) => (e.stopPropagation(), onelaborate(item.id))}
        >
          <Maximize2 size={15} />
        </button>
        {#if i > 0}
          <button
            class="ib"
            title={t("As next (Alt+Home)")}
            aria-label={t("As next")}
            onclick={(e) => (e.stopPropagation(), void run({ kind: "to_next", id: item.id }))}
          >
            <ListStart size={16} />
          </button>
        {/if}
        <button
          class="ib run"
          title={t("Take over now (Enter)")}
          aria-label={t("Take over now")}
          onclick={(e) => (e.stopPropagation(), void start(item))}
        >
          <Play size={16} />
        </button>
      {/snippet}
    </Row>
    {#if unfolded === item.id}
      <div class="unfolded">
        {#each item.resources as r (r.id)}
          <Chip resource={r} onopen={() => open(item.id, r.id)} />
        {/each}
      </div>
    {/if}
  {/each}
  {#if marked !== null}
    <div class="slot" style="top:{marked}px" aria-hidden="true"></div>
  {/if}
</div>

{#if ghost}
  <div class="ghost" aria-hidden="true" style="left:{ghost.x}px;top:{ghost.y}px;width:{ghost.w}px;height:{ghost.h}px">
    <span class="gnum">{ghost.num}</span>
    <span class="gt">{ghost.title}</span>
  </div>
{/if}

<style>
  .list {
    position: relative;
    /* Room for the drop mark below the last line. */
    padding-bottom: 6px;
  }
  /* Where the dragged line would land: a thick line with a glow and a round end on each side. It
     lies over the list, so the rows do not move while dragging. */
  .slot {
    position: absolute;
    left: 6px;
    right: 6px;
    height: 4px;
    margin-top: -2px;
    z-index: 5;
    border-radius: var(--sx-radius-s);
    background: var(--sx-accent-fill);
    box-shadow: 0 0 10px 2px color-mix(in srgb, var(--sx-accent) 65%, transparent);
    pointer-events: none;
  }
  .slot::before,
  .slot::after {
    content: "";
    position: absolute;
    top: -3px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--sx-accent-fill);
  }
  .slot::before {
    left: -4px;
  }
  .slot::after {
    right: -4px;
  }
  .ghost {
    position: fixed;
    z-index: 3;
    display: flex;
    align-items: center;
    gap: 6px;
    box-sizing: border-box;
    padding: 0 10px 0 30px;
    background: var(--sx-surface);
    border: 1px solid var(--sx-accent);
    border-radius: var(--sx-radius-l);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    color: var(--sx-ink);
    pointer-events: none;
    opacity: 0.8;
  }
  .gnum {
    width: 18px;
    text-align: right;
    color: var(--sx-ink-2);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }
  .gt {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .links {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    border: 0;
    background: none;
    padding: 2px 4px;
    border-radius: var(--sx-radius-m);
    color: var(--sx-ink-2);
    font-size: 12px;
  }
  .links:hover,
  .links.on {
    color: var(--sx-ink);
    background: var(--sx-hover);
  }
  .unfolded {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 4px 12px 10px 36px;
    border-bottom: 1px solid var(--sx-chrome-divider);
  }
  .badge {
    white-space: nowrap;
  }
  .ib {
    width: 30px;
    height: 30px;
    display: inline-grid;
    place-items: center;
    border: 1px solid transparent;
    background: none;
    border-radius: var(--sx-radius-l);
    color: var(--sx-ink-2);
    padding: 0;
  }
  .ib:hover {
    background: var(--sx-hover);
    color: var(--sx-ink);
  }
  .ib.run {
    color: var(--sx-accent-ink);
  }
</style>
