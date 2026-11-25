<script lang="ts">
  // Delegated (docs/KONZEPT.md 6.3, plan 2.3): grouped by the person, each entry with the day it was
  // handed over, the noted date (no reminder, D2), note, checklist and resources. Actions: back to
  // focus and done.
  import { Check, Maximize2, Undo2 } from "@lucide/svelte";
  import { dateLabel } from "../format";
  import { t } from "../i18n";
  import { withKey } from "../keys";
  import type { Act, Item } from "../types";
  import Chip from "./Chip.svelte";

  let {
    items,
    run,
    open,
    onelaborate,
  }: {
    items: Item[];
    run: (change: Act, message?: string) => Promise<boolean>;
    open: (id: string, resource: string) => void;
    onelaborate: (id: string) => void;
  } = $props();

  /** People in alphabetical order, each with their entries, the longest ago first. */
  const groups = $derived.by(() => {
    const by = new Map<string, Item[]>();
    for (const item of items.filter((i) => i.state === "delegated")) {
      const name = item.delegated_to ?? "?";
      by.set(name, [...(by.get(name) ?? []), item]);
    }
    return [...by.entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([name, list]) => ({
        name,
        list: list.sort((a, b) => (a.delegated_at ?? "").localeCompare(b.delegated_at ?? "")),
      }));
  });

  /** The local day of a timestamp, as the day format wants it. */
  function dayOf(iso: string | null): string {
    if (!iso) return "";
    const d = new Date(iso);
    const pad = (n: number) => String(n).padStart(2, "0");
    return dateLabel(`${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`);
  }
</script>

{#if groups.length === 0}
  <div class="empty">
    <p><b>{t("Nothing delegated.")}</b><br />{t("Tasks you hand to someone show up here, grouped by person.")}</p>
  </div>
{:else}
  {#each groups as group (group.name)}
    <section class="person">
      <h2>@{group.name} <span class="count">{group.list.length}</span></h2>
      <ul role="list">
        {#each group.list as item (item.id)}
          <li class="entry">
            <div class="top">
              <h3>{item.title || t("Untitled")}</h3>
              <button
                class="expand"
                title={withKey(t("Elaborate"), "Ctrl+E")}
                aria-label={t("Elaborate")}
                onclick={() => onelaborate(item.id)}
              >
                <Maximize2 size={13} />
              </button>
              <span class="dates">
                {t("handed over {date}", { date: dayOf(item.delegated_at) })}
                {#if item.date}
                  · {t("noted {date}", { date: dateLabel(item.date) })}{/if}
              </span>
            </div>
            {#if item.note}<p class="note">{item.note}</p>{/if}
            {#if item.checklist.length}
              <ul class="steps" role="list">
                {#each item.checklist as step (step.id)}
                  <li class:done={step.done}>{step.done ? "☑" : "☐"} {step.text}</li>
                {/each}
              </ul>
            {/if}
            {#if item.resources.length}
              <div class="chips">
                {#each item.resources as r (r.id)}<Chip resource={r} onopen={() => open(item.id, r.id)} />{/each}
              </div>
            {/if}
            <div class="acts">
              <button onclick={() => run({ kind: "send_to", id: item.id, to: "focus" }, t("Back in focus"))}>
                <Undo2 size={14} />{t("Back to focus")}
              </button>
              <button class="primary" onclick={() => run({ kind: "finish", id: item.id }, t("Marked as done"))}>
                <Check size={14} />{t("Done")}
              </button>
            </div>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
{/if}

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .person {
    margin-top: 8px;
  }
  h2 {
    margin: 0;
    padding: 8px 16px 2px;
    font-size: 12.5px;
    font-weight: 700;
    color: var(--tq-ink-2);
    letter-spacing: 0.02em;
  }
  .count {
    font-weight: 600;
    color: var(--tq-ink-3);
    margin-left: 4px;
  }
  .entry {
    margin: 6px 14px;
    padding: 10px 12px;
    background: var(--tq-card);
    border: 1px solid var(--tq-line);
    border-radius: 10px;
  }
  .top {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 2px 8px;
    align-items: center;
  }
  .dates {
    grid-column: 1 / -1;
  }
  .expand {
    display: inline-grid;
    place-items: center;
    border: 0;
    background: none;
    color: var(--tq-ink-3);
    padding: 2px;
    border-radius: 6px;
  }
  .expand:hover {
    color: var(--tq-accent);
  }
  h3 {
    margin: 0;
    font-size: 14.5px;
    font-weight: 600;
    line-height: 1.3;
  }
  .dates {
    font-size: 11.5px;
    color: var(--tq-ink-3);
  }
  .note {
    margin: 6px 0 0;
    white-space: pre-wrap;
    font-size: 13px;
    color: var(--tq-ink-2);
    max-height: 100px;
    overflow-y: auto;
  }
  .steps {
    margin-top: 6px;
    display: grid;
    gap: 2px;
    font-size: 13px;
  }
  .steps .done {
    color: var(--tq-ink-3);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }
  .acts {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }
  .acts button {
    display: inline-flex;
    gap: 5px;
    align-items: center;
    border: 1px solid var(--tq-line-strong);
    background: var(--tq-field);
    color: var(--tq-ink);
    border-radius: 8px;
    padding: 4px 11px;
    font-size: 12.5px;
  }
  .acts button:hover {
    border-color: var(--tq-accent);
  }
  .acts .primary {
    color: var(--tq-accent);
    font-weight: 600;
  }
  .empty {
    display: grid;
    justify-items: center;
    text-align: center;
    padding: 48px 24px;
    color: var(--tq-ink-2);
  }
  .empty p {
    margin: 0;
    max-width: 34ch;
  }
  .empty b {
    color: var(--tq-ink);
  }
</style>
