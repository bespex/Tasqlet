<script lang="ts">
  // Header of the popover (D41, kit K9): app mark, tabs Fokus, Ideen, Delegiert with their counts, then
  // the tools in the order every be spex tray app uses: search, pin, settings.
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

<div class="tabs sx-chrome" role="tablist">
  <svg class="mark" viewBox="0 0 512 512" aria-hidden="true">
    <rect x="32" y="32" width="448" height="448" rx="104" fill="#b4410e" />
    <rect x="120" y="120" width="272" height="272" rx="64" fill="none" stroke="#fff8f2" stroke-width="36" />
    <path
      d="M180 262l52 52 104-116"
      fill="none"
      stroke="#fff8f2"
      stroke-width="40"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
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
  {#if onsearch}
    <button class="ib" title={withKey(t("Search"), "Ctrl+F")} aria-label={t("Search")} onclick={onsearch}>
      <Search size={16} />
    </button>
  {/if}
  {#if onpin}
    <button
      class="ib"
      class:on={pinned}
      aria-pressed={pinned}
      title={pinned
        ? t("Pinned: stays in front when you click elsewhere")
        : t("Pin: stay in front when you click elsewhere")}
      aria-label={t("Pin")}
      onclick={onpin}
    >
      <Pin size={16} />
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
    border-bottom: 1px solid var(--sx-chrome-divider);
  }
  .mark {
    width: 22px;
    height: 22px;
    flex: none;
    margin: 0 8px 0 4px;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 7px;
    border: 0;
    background: none;
    padding: 7px 11px;
    border-radius: var(--sx-radius-l);
    color: var(--sx-ink-2);
    font-weight: 500;
    white-space: nowrap;
  }
  .tab:hover {
    background: var(--sx-hover);
    color: var(--sx-ink);
  }
  .tab.on {
    background: var(--sx-surface);
    color: var(--sx-ink);
    box-shadow:
      0 0 0 1px var(--sx-line),
      0 1px 2px rgba(0, 0, 0, 0.06);
  }
  .n {
    font-size: 12px;
    font-weight: 600;
    color: var(--sx-ink-2);
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
    border-radius: var(--sx-radius-l);
    color: var(--sx-ink-2);
  }
  .ib:hover {
    background: var(--sx-hover);
    color: var(--sx-ink);
  }
  .ib.on {
    color: var(--sx-accent-ink);
    background: var(--sx-accent-soft);
  }
</style>
