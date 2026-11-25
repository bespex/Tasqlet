<script lang="ts">
  // Elaboration (docs/KONZEPT.md 6.5, plan 2.4): the popover gets twice as wide; on the left all open
  // items grouped by list, on the right the editor of the chosen one. Ctrl+E, Esc or the × close it.
  import { X } from "@lucide/svelte";
  import { t } from "../i18n";
  import { withKey } from "../keys";
  import type { Act, Item, State } from "../types";
  import ElabEditor from "./ElabEditor.svelte";

  let {
    items,
    id,
    people,
    run,
    open,
    onpick,
    onclose,
  }: {
    items: Item[];
    id: string;
    people: string[];
    run: (change: Act, message?: string) => Promise<boolean>;
    open: (itemId: string, resource: string) => void;
    onpick: (id: string) => void;
    onclose: () => void;
  } = $props();

  const item = $derived(items.find((i) => i.id === id));
  const lists: { state: State; label: () => string }[] = [
    { state: "focus", label: () => t("Focus") },
    { state: "intermezzo", label: () => t("In between") },
    { state: "idea", label: () => t("Ideas") },
    { state: "delegated", label: () => t("Delegated") },
  ];
  /** Open items by list; the chosen item stays on the left even after it became done. */
  const groups = $derived(
    lists
      .map((l) => ({
        label: l.label(),
        list: items
          .filter((i) => i.state === l.state)
          .sort(
            (a, b) =>
              a.order - b.order ||
              (l.state === "idea" ? b.created_at.localeCompare(a.created_at) : 0) ||
              a.title.localeCompare(b.title),
          ),
      }))
      .filter((g) => g.list.length),
  );
  const finished = $derived(item?.state === "done" ? item : null);
  const label = (i: Item) => i.title || i.note.split("\n")[0].replace(/^[-*•]\s*/, "") || t("Untitled");
</script>

<div class="sw">
  <aside class="left" aria-label={t("Ideas & tasks")}>
    <div class="gt">{t("IDEAS & TASKS")}</div>
    {#each groups as g (g.label)}
      <div class="gh">{g.label}</div>
      {#each g.list as it (it.id)}
        <button class="sl" class:on={it.id === id} data-drop={it.id} onclick={() => onpick(it.id)} title={label(it)}>
          {label(it)}
        </button>
      {/each}
    {/each}
    {#if finished}
      <div class="gh">{t("Done")}</div>
      <button class="sl on" title={label(finished)}>{label(finished)}</button>
    {/if}
  </aside>
  <section class="right" data-drop={item?.id}>
    <div class="top">
      <span class="gt">{t("ELABORATION")}</span>
      <span class="sp"></span>
      <kbd>Esc</kbd>
      <button class="x" title={withKey(t("Close"), "Ctrl+E")} aria-label={t("Close")} onclick={onclose}>
        <X size={16} />
      </button>
    </div>
    {#if item}
      {#key item.id}
        <ElabEditor {item} {people} {run} open={(resource) => open(item.id, resource)} />
      {/key}
    {/if}
  </section>
</div>

<style>
  .sw {
    display: grid;
    grid-template-columns: 250px minmax(0, 1fr);
    height: 100%;
    min-height: 0;
  }
  .left {
    overflow-y: auto;
    padding: 14px 8px;
    border-right: 1px solid var(--sx-line);
    background: var(--sx-field);
  }
  .gt {
    padding: 0 6px;
    font-size: 12px;
    font-weight: 700;
    color: var(--sx-ink-2);
    letter-spacing: 0.03em;
  }
  .gh {
    margin: 14px 6px 4px;
    font-size: 12px;
    color: var(--sx-ink-2);
  }
  .sl {
    display: block;
    width: 100%;
    text-align: left;
    border: 0;
    background: none;
    color: var(--sx-ink);
    font: inherit;
    font-size: 13px;
    padding: 6px 8px;
    border-radius: var(--sx-radius-m);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sl:hover {
    background: var(--sx-hover, var(--sx-surface));
  }
  .sl.on {
    background: var(--sx-surface);
    box-shadow: 0 0 0 1px var(--sx-line);
    font-weight: 600;
  }
  .right {
    overflow-y: auto;
    padding: 14px 24px 28px;
    min-width: 0;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .sp {
    flex: 1;
  }
  kbd {
    font-size: 11px;
    color: var(--sx-ink-3);
    border: 1px solid var(--sx-line);
    border-radius: var(--sx-radius-s);
    padding: 0 5px;
  }
  .x {
    display: inline-grid;
    place-items: center;
    border: 1px solid transparent;
    background: none;
    color: var(--sx-ink-2);
    border-radius: var(--sx-radius-l);
    padding: 4px;
  }
  .x:hover {
    color: var(--sx-accent-ink);
    border-color: var(--sx-line-strong);
  }
</style>
