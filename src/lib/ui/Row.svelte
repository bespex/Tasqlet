<script lang="ts">
  // One line of the queue (docs/KONZEPT.md 6.1): grip, number, title, indicators; the tools only
  // appear on hover, selection or keyboard focus. The queue around it is a `listbox`; the keys
  // (↑/↓, Enter, Alt+↑/↓) come with plan 1.8.
  import type { Snippet } from "svelte";
  import { GripVertical } from "@lucide/svelte";
  import { t } from "../i18n";

  let {
    num,
    title,
    selected = false,
    indicators,
    tools,
    onselect,
  }: {
    num: number;
    title: string;
    selected?: boolean;
    indicators?: Snippet;
    tools?: Snippet;
    onselect?: () => void;
  } = $props();
</script>

<div
  class="row"
  class:sel={selected}
  role="option"
  aria-selected={selected}
  tabindex="0"
  onclick={onselect}
  onkeydown={() => {}}
>
  <span class="grip" title={t("Drag to sort (Alt+↑/↓)")}><GripVertical size={16} /></span>
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
  .grip {
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
  .row:focus-within .tools,
  .row:focus-within .grip {
    opacity: 1;
  }
</style>
