<script lang="ts">
  // A section of the work card (docs/KONZEPT.md 6.1): label on the left with "+" below it to add,
  // content on the right, separated from the previous section by a fine line.
  import type { Snippet } from "svelte";
  import { Plus } from "@lucide/svelte";
  import { t } from "../i18n";

  let { label, onadd, children }: { label: string; onadd?: () => void; children: Snippet } = $props();
</script>

<section class="f">
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
    margin-top: 14px;
    padding-top: 14px;
    border-top: 1px solid var(--tq-line);
    align-items: start;
  }
  .lbl {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    color: var(--tq-ink-2);
    font-size: 12.5px;
    font-weight: 600;
    padding-top: 3px;
  }
  .plus {
    width: 24px;
    height: 24px;
    display: inline-grid;
    place-items: center;
    border: 1px solid var(--tq-line);
    background: var(--tq-card);
    border-radius: 50%;
    color: var(--tq-ink-2);
    padding: 0;
  }
  .plus:hover {
    color: var(--tq-on-accent);
    background: var(--tq-accent);
    border-color: var(--tq-accent);
  }
</style>
