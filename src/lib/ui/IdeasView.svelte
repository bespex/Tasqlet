<script lang="ts">
  // Ideas (docs/KONZEPT.md 6.2, plan 2.2): cards, newest first, with a quiet age. Each shows at most
  // three bullets of the note and "+n more", its resources and the checklist progress. A line at the
  // bottom adds a brainstorm thought with Enter (a link becomes a resource); the buttons send the idea
  // on: "→ Focus" and "⚡ In between".
  import { Link, Maximize2 } from "@lucide/svelte";
  import { ageLabel, bullets } from "../format";
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

  const ideas = $derived(
    items.filter((i) => i.state === "idea").sort((a, b) => b.created_at.localeCompare(a.created_at)),
  );
  let drafts = $state<Record<string, string>>({});

  /** The title, or the first bullet if the idea has none (it then does not repeat in the bullets). */
  const parts = (item: Item) => {
    const all = bullets(item.note);
    const untitled = !item.title.trim();
    return { title: untitled ? (all[0] ?? t("Untitled")) : item.title, rest: untitled ? all.slice(1) : all };
  };

  async function add(item: Item) {
    const text = (drafts[item.id] ?? "").trim();
    if (text && (await run({ kind: "brainstorm", id: item.id, text }))) drafts[item.id] = "";
  }
</script>

{#if ideas.length === 0}
  <div class="empty">
    <p><b>{t("No ideas yet.")}</b><br />{t("⏎ in the quick entry saves a thought as idea.")}</p>
  </div>
{:else}
  <ul role="list">
    {#each ideas as item (item.id)}
      {@const p = parts(item)}
      <li class="idea" data-drop={item.id}>
        <div class="top">
          <h3>{p.title}</h3>
          <span class="age">{ageLabel(item.created_at)}</span>
          <button
            class="expand"
            title={withKey(t("Elaborate"), "Ctrl+E")}
            aria-label={t("Elaborate")}
            onclick={() => onelaborate(item.id)}
          >
            <Maximize2 size={13} />
          </button>
        </div>
        {#if p.rest.length}
          <ul class="pts" role="list">
            {#each p.rest.slice(0, 3) as point, i (i)}<li>{point}</li>{/each}
          </ul>
          {#if p.rest.length > 3}<div class="more">{t("{n} more", { n: p.rest.length - 3 })}</div>{/if}
        {/if}
        {#if item.resources.length || item.checklist.length}
          <div class="meta">
            {#each item.resources.slice(0, 3) as r (r.id)}
              <Chip resource={r} onopen={() => open(item.id, r.id)} />
            {/each}
            {#if item.resources.length > 3}<span class="n"><Link size={12} />+{item.resources.length - 3}</span>{/if}
            {#if item.checklist.length}
              <span class="n">{item.checklist.filter((c) => c.done).length}/{item.checklist.length}</span>
            {/if}
          </div>
        {/if}
        <input
          class="brain"
          bind:value={drafts[item.id]}
          placeholder={t("+ Add a brainstorm thought …")}
          aria-label={t("Add a brainstorm thought")}
          onkeydown={(e) => {
            if (e.key === "Enter" && !e.isComposing) void add(item);
          }}
        />
        <div class="acts">
          <button
            class="primary"
            onclick={() => run({ kind: "send_to", id: item.id, to: "focus" }, t("Moved to focus"))}
          >
            {t("→ Focus")}
          </button>
          <button onclick={() => run({ kind: "send_to", id: item.id, to: "intermezzo" }, t("Put into “In between”"))}>
            {t("⚡ In between")}
          </button>
        </div>
      </li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .idea {
    margin: 10px 14px;
    padding: 10px 12px 10px;
    background: var(--sx-surface);
    border: 1px solid var(--sx-line);
    border-radius: var(--sx-radius-l);
  }
  .top {
    display: flex;
    gap: 10px;
    align-items: baseline;
  }
  h3 {
    flex: 1;
    margin: 0;
    font-size: 14.5px;
    font-weight: 600;
    line-height: 1.3;
  }
  .age {
    font-size: 11.5px;
    color: var(--sx-ink-3);
    white-space: nowrap;
  }
  .expand {
    display: inline-grid;
    place-items: center;
    border: 0;
    background: none;
    color: var(--sx-ink-3);
    padding: 2px;
    border-radius: var(--sx-radius-m);
    align-self: center;
  }
  .expand:hover {
    color: var(--sx-accent-ink);
  }
  .pts {
    margin-top: 6px;
    display: grid;
    gap: 2px;
    font-size: 13px;
    color: var(--sx-ink-2);
  }
  .pts li::before {
    content: "• ";
  }
  .more {
    margin-top: 2px;
    font-size: 12px;
    color: var(--sx-ink-3);
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    margin-top: 8px;
  }
  .n {
    display: inline-flex;
    gap: 3px;
    align-items: center;
    font-size: 12px;
    color: var(--sx-ink-2);
  }
  .brain {
    width: 100%;
    box-sizing: border-box;
    margin-top: 8px;
    font: inherit;
    font-size: 13px;
    background: var(--sx-field);
    color: var(--sx-ink);
    border: 1px dashed var(--sx-line-strong);
    border-radius: var(--sx-radius-l);
    padding: 5px 8px;
  }
  .brain:focus {
    outline: 0;
    border-style: solid;
    border-color: var(--sx-accent);
  }
  .acts {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
  .acts button {
    border: 1px solid var(--sx-line-strong);
    background: var(--sx-field);
    color: var(--sx-ink);
    border-radius: var(--sx-radius-l);
    padding: 4px 11px;
    font-size: 12.5px;
  }
  .acts button:hover {
    border-color: var(--sx-accent);
  }
  .acts .primary {
    color: var(--sx-accent-ink);
    font-weight: 600;
  }
  .empty {
    display: grid;
    justify-items: center;
    text-align: center;
    padding: 48px 24px;
    color: var(--sx-ink-2);
  }
  .empty p {
    margin: 0;
    max-width: 34ch;
  }
  .empty b {
    color: var(--sx-ink);
  }
</style>
