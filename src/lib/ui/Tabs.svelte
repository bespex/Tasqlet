<script lang="ts">
  // Reiter: Fokus, Ideen, Delegiert with their counts, the search at the end (docs/KONZEPT.md 6).
  import { Pin, Search, Settings } from "@lucide/svelte";
  import { t } from "../i18n";
  import { withKey } from "../keys";

  type Tab = { id: string; label: string; count: number };

  let {
    tabs,
    active,
    onselect,
    onsearch,
    onsettings,
    pinned = false,
    onpin,
  }: {
    tabs: Tab[];
    active: string;
    onselect: (id: string) => void;
    onsearch?: () => void;
    onsettings?: () => void;
    pinned?: boolean;
    onpin?: () => void;
  } = $props();
</script>

<div class="tabs zone-chrome" role="tablist">
  {#each tabs as tab, i (tab.id)}
    <button
      class="tab"
      title={withKey(tab.label, `Ctrl+${i + 1}`)}
      class:on={tab.id === active}
      role="tab"
      aria-selected={tab.id === active}
      onclick={() => onselect(tab.id)}
    >
      {tab.label}
      <span class="n">{tab.count}</span>
    </button>
  {/each}
  <span class="sp"></span>
  {#if onpin}
    <button
      class="ib"
      class:on={pinned}
      aria-pressed={pinned}
      title={pinned ? t("Pinned: stays open when you click elsewhere") : t("Pin: stay open when you click elsewhere")}
      aria-label={t("Pin")}
      onclick={onpin}
    >
      <Pin size={16} />
    </button>
  {/if}
  {#if onsearch}
    <button class="ib" title={withKey(t("Search"), "Ctrl+F")} aria-label={t("Search")} onclick={onsearch}>
      <Search size={16} />
    </button>
  {/if}
  {#if onsettings}
    <button class="ib" title={t("Settings")} aria-label={t("Settings")} onclick={onsettings}>
      <Settings size={16} />
    </button>
  {/if}
</div>

<style>
  .tabs {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 10px 10px 8px;
    border-bottom: 1px solid var(--tq-c-divider);
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 7px;
    border: 0;
    background: none;
    padding: 7px 11px;
    border-radius: 8px;
    color: var(--tq-ink-2);
    font-weight: 500;
    white-space: nowrap;
  }
  .tab:hover {
    background: var(--tq-hover);
    color: var(--tq-ink);
  }
  .tab.on {
    background: var(--tq-card);
    color: var(--tq-ink);
    box-shadow:
      0 0 0 1px var(--tq-line),
      0 1px 2px rgba(0, 0, 0, 0.06);
  }
  .n {
    font-size: 12px;
    font-weight: 600;
    color: var(--tq-ink-2);
    font-variant-numeric: tabular-nums;
  }
  .sp {
    flex: 1;
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
  }
  .ib:hover {
    background: var(--tq-hover);
    color: var(--tq-ink);
  }
  .ib.on {
    color: var(--tq-accent);
    background: var(--tq-accent-soft);
  }
</style>
