<script lang="ts">
  // The queue #2 … n (docs/KONZEPT.md 6.1, 6.6): one line per task. Keys: ↑/↓ select (↑ on the first
  // line back to the quick entry), Enter takes over, Alt+↑/↓ move, Alt+Home next. Sorting by pointer
  // goes through the grip (no HTML5 drag and drop, which WebKitGTK handles badly).
  import { tick } from "svelte";
  import { ListStart, Play } from "@lucide/svelte";
  import { t } from "../i18n";
  import { dropIndex, moveDelta } from "../queue";
  import type { Act, Item } from "../types";
  import Row from "./Row.svelte";

  let {
    items,
    run,
    onup,
  }: {
    items: Item[];
    run: (change: Act, message?: string) => Promise<boolean>;
    /** ↑ on the first line: back into the quick entry. */
    onup?: () => void;
  } = $props();

  let list: HTMLElement;
  let selected = $state<string | null>(null);
  let drag = $state<{ id: string; from: number; drop: number } | null>(null);

  const rows = () => [...list.querySelectorAll<HTMLElement>("[role=option]")];

  /** Focus the first line (from the quick entry's ↓). */
  export function enter() {
    rows()[0]?.focus();
  }

  /** The selected task if one of the lines has the focus, e.g. as target of pasting (6.6). */
  export function target(): string | null {
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
    drag = { id, from, drop: from };
    const middles = () =>
      rows().map((r) => {
        const b = r.getBoundingClientRect();
        return b.top + b.height / 2;
      });
    const over = (ev: PointerEvent) => {
      if (drag) drag.drop = dropIndex(ev.clientY, middles());
    };
    const end = async (ev: PointerEvent) => {
      window.removeEventListener("pointermove", over);
      window.removeEventListener("pointerup", end);
      window.removeEventListener("pointercancel", end);
      over(ev);
      const done = drag;
      drag = null;
      const delta = done && ev.type === "pointerup" ? moveDelta(done.from, done.drop) : 0;
      if (delta) await move(items[from], { kind: "move_by", id, delta });
    };
    window.addEventListener("pointermove", over);
    window.addEventListener("pointerup", end);
    window.addEventListener("pointercancel", end);
  }

  /** Where the drop line goes for row `i`, if the drag would actually move something. */
  function line(i: number): "before" | "after" | undefined {
    if (!drag || moveDelta(drag.from, drag.drop) === 0) return undefined;
    if (drag.drop === i) return "before";
    if (drag.drop === items.length && i === items.length - 1) return "after";
    return undefined;
  }
</script>

<div role="listbox" tabindex="-1" aria-label={t("Focus")} bind:this={list} {onkeydown}>
  {#each items as item, i (item.id)}
    <Row
      id={item.id}
      num={i + 2}
      title={item.title}
      selected={item.id === selected}
      dragging={drag?.id === item.id}
      line={line(i)}
      onselect={() => (selected = item.id)}
      ongrip={(e) => startDrag(e, item.id)}
    >
      {#snippet indicators()}
        {#if item.last_active}<span class="badge">{t("↩ last worked on")}</span>{/if}
      {/snippet}
      {#snippet tools()}
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
  {/each}
</div>

<style>
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
    border-radius: 8px;
    color: var(--tq-ink-2);
    padding: 0;
  }
  .ib:hover {
    background: var(--tq-hover);
    color: var(--tq-ink);
  }
  .ib.run {
    color: var(--tq-accent);
  }
</style>
