<script lang="ts">
  // The bar for small things in between (docs/KONZEPT.md 6.1, plan 2.1): only a ⚡ as mark, the name and
  // the count are in its tooltip, then pills (three, then "+n"). A click takes the task over, a middle
  // click or the ✓ that shows on hover ticks it off.
  import { Check, Zap } from "@lucide/svelte";
  import { t } from "../i18n";
  import type { Act, Item } from "../types";

  let {
    items,
    run,
  }: {
    items: Item[];
    run: (change: Act, message?: string) => Promise<boolean>;
  } = $props();

  const MAX = 3;
  let all = $state(false);

  const list = $derived(items.filter((i) => i.state === "intermezzo").sort((a, b) => a.order - b.order));
  const shown = $derived(all ? list : list.slice(0, MAX));

  const label = (item: Item) => item.title || t("Untitled");
  const takeOver = (item: Item) => run({ kind: "start", id: item.id }, t("Now in progress"));
  const tick = (item: Item) => run({ kind: "finish", id: item.id }, t("Ticked off: {title}", { title: label(item) }));
</script>

<div class="inter sx-chrome">
  <span class="il" title={`${t("In between")}: ${list.length}`} aria-label={`${t("In between")}, ${list.length}`}>
    <Zap size={15} />
  </span>
  <div class="pills">
    {#each shown as item (item.id)}
      <span class="pill" data-drop={item.id}>
        <button
          class="pb"
          title={`${label(item)}\n${t("Click: take over now · Middle click: tick off")}`}
          onclick={() => takeOver(item)}
          onmousedown={(e) => e.button === 1 && e.preventDefault()}
          onauxclick={(e) => {
            if (e.button === 1) {
              e.preventDefault();
              void tick(item);
            }
          }}
        >
          {label(item)}
        </button>
        <button class="pd" title={t("Tick off")} aria-label={t("Tick off")} onclick={() => tick(item)}>
          <Check size={14} />
        </button>
      </span>
    {/each}
    {#if !list.length}<span class="empty">{t("empty · Alt ⏎ puts something here")}</span>{/if}
  </div>
  {#if list.length > MAX}
    <button class="pill more" aria-expanded={all} onclick={() => (all = !all)}>
      {all ? "−" : `+${list.length - MAX}`}
    </button>
  {/if}
</div>

<style>
  .inter {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-top: 1px solid var(--sx-line);
    background: var(--sx-bg);
  }
  .il {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--sx-accent-soft);
    color: var(--sx-accent-ink);
    flex: none;
  }
  .pills {
    display: flex;
    gap: 6px;
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    flex: 0 1 auto;
    min-width: 0;
    border: 1px solid var(--sx-line);
    background: var(--sx-surface);
    border-radius: var(--sx-radius-l);
    white-space: nowrap;
  }
  .pill:hover {
    border-color: var(--sx-line-strong);
  }
  .pb {
    border: 0;
    background: none;
    color: var(--sx-ink);
    padding: 4px 12px;
    font-size: 13px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  /* The ✓ takes room only while hovering, so resting pills stay compact. */
  .pd {
    display: none;
    border: 0;
    background: none;
    padding: 4px 9px 4px 0;
    color: var(--sx-ink-3);
    place-items: center;
  }
  .pill:hover .pb,
  .pill:has(:focus-visible) .pb {
    padding-right: 4px;
  }
  .pill:hover .pd,
  .pill:has(:focus-visible) .pd {
    display: inline-grid;
  }
  .pd:hover {
    color: var(--sx-accent-ink);
  }
  .more {
    flex: none;
    border: 1px solid var(--sx-line);
    background: var(--sx-surface);
    border-radius: var(--sx-radius-l);
    padding: 4px 11px;
    font-size: 13px;
    color: var(--sx-ink-2);
  }
  .empty {
    align-self: center;
    font-size: 13px;
    color: var(--sx-ink-2);
  }
</style>
