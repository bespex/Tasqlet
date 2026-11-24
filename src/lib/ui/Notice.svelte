<script lang="ts">
  // A calm hint at the top of the popover: no tray found, data not loadable. Not a dialog: Tasqlet
  // keeps working beneath it.
  import { Info, TriangleAlert } from "@lucide/svelte";

  let {
    title,
    text,
    warn = false,
    action,
    onaction,
  }: { title: string; text: string; warn?: boolean; action?: string; onaction?: () => void } = $props();
</script>

<div class="notice" class:warn role={warn ? "alert" : "note"}>
  <span class="ic">
    {#if warn}<TriangleAlert size={16} />{:else}<Info size={16} />{/if}
  </span>
  <div class="tx">
    <b>{title}</b>
    <p>{text}</p>
    {#if action && onaction}<button class="btn" onclick={onaction}>{action}</button>{/if}
  </div>
</div>

<style>
  .notice {
    display: flex;
    gap: 10px;
    margin: 12px 14px 0;
    padding: 10px 12px;
    border: 1px solid var(--tq-line-strong);
    border-radius: 10px;
    background: var(--tq-card);
    font-size: 13px;
    color: var(--tq-ink-2);
  }
  .notice.warn {
    border-color: var(--tq-accent);
    background: var(--tq-accent-soft);
    color: var(--tq-ink);
  }
  .ic {
    color: var(--tq-accent);
    padding-top: 1px;
  }
  .tx {
    display: grid;
    gap: 4px;
  }
  b {
    color: var(--tq-ink);
  }
  p {
    margin: 0;
  }
  .btn {
    justify-self: start;
    margin-top: 4px;
    border: 1px solid var(--tq-line-strong);
    background: var(--tq-card);
    color: var(--tq-ink);
    border-radius: 8px;
    padding: 5px 10px;
    font-size: 13px;
    font-weight: 500;
  }
  .btn:hover {
    background: var(--tq-hover);
  }
</style>
