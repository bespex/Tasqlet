<script lang="ts">
  // Chip of a resource (docs/KONZEPT.md 5): symbol by type, the label, the fixed Alt+n digit, the full
  // target on hover, and an "x" to remove it while hovering. Without a resource it is the "…" chip.
  import { Ellipsis, File, Link, Mail, X } from "@lucide/svelte";
  import { t } from "../i18n";
  import type { Resource } from "../types";

  let {
    resource,
    digit,
    onopen,
    onremove,
  }: {
    resource?: Resource;
    digit?: number | null;
    onopen?: () => void;
    onremove?: () => void;
  } = $props();

  const label = $derived(resource ? (resource.shown ?? resource.label ?? resource.target) : "");
</script>

{#if resource}
  <span class="chip" title={resource.target}>
    <button class="open" onclick={onopen}>
      {#if resource.type === "url"}<Link size={14} />{:else if resource.type === "mail"}<Mail size={14} />{:else}<File
          size={14}
        />{/if}
      <span class="lb">{label}</span>
      {#if digit}<span class="kn">{digit}</span>{/if}
    </button>
    {#if onremove}
      <button class="x" title={t("Remove")} aria-label={t("Remove")} onclick={onremove}><X size={12} /></button>
    {/if}
  </span>
{:else}
  <button class="chip more" title={t("All resources")} aria-label={t("All resources")} onclick={onopen}>
    <Ellipsis size={14} />
  </button>
{/if}

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    border: 1px solid var(--tq-line);
    background: var(--tq-field);
    color: var(--tq-ink);
    border-radius: 999px;
    padding: 0 6px 0 0;
    font-size: 13px;
    max-width: 100%;
    white-space: nowrap;
  }
  .chip:hover {
    border-color: var(--tq-line-strong);
    background: var(--tq-hover);
  }
  .open {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    background: none;
    padding: 4px 4px 4px 10px;
    color: inherit;
    min-width: 0;
  }
  .open :global(svg) {
    color: var(--tq-ink-2);
    flex: none;
  }
  .lb {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .kn {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--tq-ink-2);
    border: 1px solid var(--tq-line);
    border-radius: 4px;
    padding: 0 4px;
  }
  .x {
    display: none;
    border: 0;
    background: none;
    padding: 2px;
    color: var(--tq-ink-2);
    border-radius: 50%;
  }
  .chip:hover .x,
  .x:focus-visible {
    display: inline-grid;
  }
  .x:hover {
    color: var(--tq-ink);
  }
  .chip.more {
    background: none;
    padding: 4px 8px;
    color: var(--tq-ink-2);
  }
</style>
