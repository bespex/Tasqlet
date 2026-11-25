<script lang="ts">
  // One line of the queue (docs/KONZEPT.md 6.1): grip, number, title, indicators; the tools only
  // appear on hover, selection or keyboard focus. The queue around it (`Queue.svelte`) is a `listbox`
  // and handles the keys.
  import type { Snippet } from "svelte";
  import { GripVertical } from "@lucide/svelte";
  import { t } from "../i18n";

  let {
    id,
    num,
    title,
    selected = false,
    dragging = false,
    indicators,
    tools,
    onselect,
    ongrip,
  }: {
    id: string;
    num: number;
    title: string;
    selected?: boolean;
    /** This row is being dragged. */
    dragging?: boolean;
    indicators?: Snippet;
    tools?: Snippet;
    onselect?: () => void;
    /** Pointer went down on the grip (sorting by pointer, not by HTML5 drag and drop). */
    ongrip?: (e: PointerEvent) => void;
  } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="row"
  class:sel={selected}
  class:dragging
  data-id={id}
  data-drop={id}
  role="option"
  aria-selected={selected}
  tabindex={selected ? 0 : -1}
  onclick={onselect}
  onfocus={onselect}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <span class="grip" title={t("Drag to sort (Alt+↑/↓)")} onpointerdown={ongrip}><GripVertical size={16} /></span>
  <span class="num">{num}</span>
  <span class="rt" {title}>{title}</span>
  {#if indicators}<span class="ind">{@render indicators()}</span>{/if}
  {#if tools}<span class="tools">{@render tools()}</span>{/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px 0 6px;
    height: 44px;
    border-bottom: 1px solid var(--tq-line);
    outline: 0;
    color: var(--tq-row-ink);
  }
  .row:hover,
  .row.sel {
    background: var(--tq-hover);
    color: var(--tq-ink);
  }
  .row:focus-visible,
  .row.sel {
    box-shadow: inset 2px 0 0 var(--tq-accent);
  }
  .row {
    position: relative;
  }
  .row.dragging {
    opacity: 0.45;
  }
  .grip {
    touch-action: none;
    color: var(--tq-ink-3);
    cursor: grab;
    display: grid;
    place-items: center;
    width: 18px;
    opacity: 0;
  }
  .num {
    width: 18px;
    text-align: right;
    color: var(--tq-ink-2);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }
  .rt {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ind {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 12px;
    color: var(--tq-ink-2);
  }
  .tools {
    display: flex;
    opacity: 0;
  }
  .row:hover .tools,
  .row:hover .grip,
  .row.sel .tools,
  .row.sel .grip,
  .row:has(:focus-visible) .tools,
  .row:has(:focus-visible) .grip {
    opacity: 1;
  }
</style>
