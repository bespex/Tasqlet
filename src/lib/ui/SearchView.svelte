<script lang="ts">
  // The search (docs/KONZEPT.md 6.4, plan 1.10): a field, below it the hits with the words marked.
  // Without words it shows what was finished lately. A click unfolds a simple detail view with the
  // actions "Take over" and "Back to focus"; the full elaboration comes with plan 2.4.
  import { onMount } from "svelte";
  import { File, Link, Mail, Search as SearchIcon, X } from "@lucide/svelte";
  import { search } from "../backend";
  import { dateLabel } from "../format";
  import { t } from "../i18n";
  import { searchTerms } from "../mock-search";
  import type { Act, Item, SearchResult } from "../types";
  import Chip from "./Chip.svelte";
  import Mark from "./Mark.svelte";

  let {
    version,
    run,
    open,
    onclose,
  }: {
    /** Changes whenever the items changed, so the hits are looked up again. */
    version: unknown;
    run: (change: Act, message?: string) => Promise<boolean>;
    open: (id: string, resource: string) => void;
    onclose: () => void;
  } = $props();

  let query = $state("");
  let result = $state<SearchResult>({ recent: true, items: [] });
  let unfolded = $state<string | null>(null);
  let field: HTMLInputElement;
  let asked = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  const terms = $derived(searchTerms(query));

  async function lookup() {
    const mine = ++asked;
    const found = await search(query);
    if (mine === asked) result = found;
  }

  // New words: after a short pause, so typing does not ask for every key. Changed items: at once.
  $effect(() => {
    void query;
    clearTimeout(timer);
    timer = setTimeout(lookup, 80);
    return () => clearTimeout(timer);
  });
  $effect(() => {
    void version;
    void lookup();
  });
  onMount(() => field.focus());

  const dayOf = (iso: string | null) => {
    if (!iso) return "";
    const d = new Date(iso);
    const pad = (n: number) => String(n).padStart(2, "0");
    return dateLabel(`${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`);
  };

  function where(item: Item): string {
    switch (item.state) {
      case "focus":
        return item.active ? t("In progress") : t("Focus");
      case "intermezzo":
        return t("In between");
      case "idea":
        return t("Idea");
      case "delegated":
        return `${t("Delegated")} @${item.delegated_to ?? ""}`;
      case "done":
        return `${t("Done")} ${dayOf(item.done_at)}`;
    }
  }

  /** The first line of the note or the checklist that holds one of the words, if the title does not. */
  function snippet(item: Item): string {
    const words = terms.map((w) => w.replace(/^@/, "")).filter(Boolean);
    if (!words.length || words.some((w) => item.title.toLowerCase().includes(w))) return "";
    const lines = [...item.note.split("\n"), ...item.checklist.map((c) => c.text)];
    const line = lines.find((l) => words.some((w) => l.toLowerCase().includes(w)));
    return line ? (line.length > 110 ? `${line.slice(0, 110)} …` : line.trim()) : "";
  }

  const takeOver = (item: Item) => run({ kind: "start", id: item.id }, t("Now in progress"));
  const backToFocus = (item: Item) => run({ kind: "send_to", id: item.id, to: "focus" }, t("Back in focus"));
</script>

<div class="sv">
  <label class="box">
    <SearchIcon size={16} />
    <input
      bind:this={field}
      bind:value={query}
      autocomplete="off"
      spellcheck="false"
      placeholder={t("Search everything, finished items too …")}
      aria-label={t("Search")}
    />
    <button class="x" title={`${t("Close search")} (Esc)`} aria-label={t("Close search")} onclick={onclose}>
      <X size={15} />
    </button>
  </label>

  <div class="head">
    {#if result.recent}
      {t("Recently finished")}
    {:else}
      {t("{n} hits", { n: result.items.length })}
    {/if}
  </div>

  {#if result.items.length === 0}
    <p class="none">{result.recent ? t("Nothing finished yet.") : t("Nothing found.")}</p>
  {/if}

  <ul role="list">
    {#each result.items as item (item.id)}
      <li class:open={unfolded === item.id}>
        <button
          class="hit"
          aria-expanded={unfolded === item.id}
          onclick={() => (unfolded = unfolded === item.id ? null : item.id)}
        >
          <span class="t"><Mark text={item.title || t("Untitled")} {terms} /></span>
          <span class="w" class:done={item.state === "done"}>{where(item)}</span>
          {#if snippet(item)}<span class="s"><Mark text={snippet(item)} {terms} /></span>{/if}
        </button>
        {#if unfolded === item.id}
          <div class="detail">
            {#if item.note}<p class="note"><Mark text={item.note} {terms} /></p>{/if}
            {#if item.checklist.length}
              <ul class="steps" role="list">
                {#each item.checklist as step (step.id)}
                  <li class:done={step.done}>{step.done ? "☑" : "☐"} <Mark text={step.text} {terms} /></li>
                {/each}
              </ul>
            {/if}
            {#if item.resources.length}
              <div class="chips">
                {#each item.resources as r (r.id)}
                  <Chip resource={r} onopen={() => open(item.id, r.id)} />
                {/each}
              </div>
            {/if}
            <div class="acts">
              {#if !item.active}
                <button class="primary" onclick={() => takeOver(item)}>{t("Take over")}</button>
              {/if}
              {#if item.state !== "focus"}
                <button onclick={() => backToFocus(item)}>{t("Back to focus")}</button>
              {/if}
            </div>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
</div>

<style>
  .sv {
    padding: 8px 0 12px;
  }
  .box {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 6px 14px 0;
    padding: 0 10px;
    height: 38px;
    background: var(--tq-field);
    border: 1px solid var(--tq-accent);
    border-radius: 10px;
    color: var(--tq-ink-2);
  }
  .box input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--tq-ink);
    font: inherit;
  }
  .x {
    display: grid;
    place-items: center;
    border: 0;
    background: none;
    color: var(--tq-ink-2);
    padding: 2px;
    border-radius: 6px;
  }
  .x:hover {
    color: var(--tq-ink);
    background: var(--tq-hover);
  }
  .head {
    padding: 12px 16px 4px;
    font-size: 12px;
    font-weight: 600;
    color: var(--tq-ink-2);
  }
  .none {
    margin: 18px 16px;
    color: var(--tq-ink-3);
    font-size: 13.5px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .sv > ul > li {
    border-bottom: 1px solid var(--tq-c-divider);
  }
  .hit {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 10px;
    width: 100%;
    text-align: left;
    border: 0;
    background: none;
    color: var(--tq-ink);
    padding: 9px 16px;
    font: inherit;
  }
  .hit:hover,
  li.open .hit {
    background: var(--tq-hover);
  }
  .t {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .w {
    font-size: 11.5px;
    color: var(--tq-ink-2);
    white-space: nowrap;
    align-self: center;
  }
  .w.done {
    color: var(--tq-ink-3);
  }
  .s {
    grid-column: 1 / -1;
    font-size: 12.5px;
    color: var(--tq-ink-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .detail {
    padding: 4px 16px 12px 24px;
    display: grid;
    gap: 8px;
    background: var(--tq-hover);
  }
  .note {
    margin: 0;
    white-space: pre-wrap;
    font-size: 13.5px;
    line-height: 1.45;
    max-height: 120px;
    overflow-y: auto;
  }
  .steps {
    display: grid;
    gap: 2px;
    font-size: 13.5px;
  }
  .steps .done {
    color: var(--tq-ink-3);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .acts {
    display: flex;
    gap: 8px;
  }
  .acts button {
    border: 1px solid var(--tq-line-strong);
    background: var(--tq-card);
    color: var(--tq-ink);
    border-radius: 8px;
    padding: 5px 12px;
    font-size: 13px;
  }
  .acts button:hover {
    border-color: var(--tq-accent);
  }
  .acts .primary {
    background: var(--tq-accent);
    border-color: var(--tq-accent);
    color: var(--tq-on-accent);
    font-weight: 600;
  }
</style>
