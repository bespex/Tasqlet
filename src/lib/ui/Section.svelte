<script lang="ts">
  // A section of the work card (docs/KONZEPT.md 6.1): label on the left with "+" below it to add,
  // content on the right, separated from the previous section by a fine line. Empty, it takes one
  // line: the "+" sits right of the label and no "none yet" is needed.
  import type { Snippet } from "svelte";
  import { Plus } from "@lucide/svelte";
  import { t } from "../i18n";

  let {
    label,
    onadd,
    empty = false,
    children,
  }: { label: string; onadd?: () => void; empty?: boolean; children: Snippet } = $props();
</script>

<section class="f" class:empty>
  <div class="lbl">
    {label}
    {#if onadd}
      <button class="plus" title={t("Add")} aria-label={`${label}: ${t("Add")}`} onclick={onadd}>
        <Plus size={13} />
      </button>
    {/if}
  </div>
  <div class="content">{@render children()}</div>
</section>

<style>
  .f {
    display: grid;
    grid-template-columns: 96px minmax(0, 1fr);
    gap: 10px;
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--sx-line);
    align-items: start;
  }
  .lbl {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    color: var(--sx-ink-2);
    font-size: 12.5px;
    font-weight: 600;
    padding-top: 3px;
  }
  .empty {
    grid-template-columns: auto minmax(0, 1fr);
  }
  .empty .lbl {
    flex-direction: row;
    align-items: center;
    padding-top: 0;
  }
  .plus {
    width: 24px;
    height: 24px;
    display: inline-grid;
    place-items: center;
    border: 1px solid var(--sx-line);
    background: var(--sx-surface);
    border-radius: 50%;
    color: var(--sx-ink-2);
    padding: 0;
  }
  .plus:hover {
    color: var(--sx-on-accent);
    background: var(--sx-accent-fill);
    border-color: var(--sx-accent);
  }
</style>
