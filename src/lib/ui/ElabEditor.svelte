<script lang="ts">
  // The right side of the elaboration (docs/KONZEPT.md 6.5, plan 2.4): title, status, date, delegate,
  // then resources, checklist and note with "+" at the headings. Any item from any list can be edited
  // completely here. One editor per item (the view keys it by id), so drafts never leak into the next.
  // No native <select> or date picker: their pop-ups are windows of their own and would close the popover.
  import { onDestroy } from "svelte";
  import { Check, Plus, X } from "@lucide/svelte";
  import { dateLabel, parseDay } from "../format";
  import { t } from "../i18n";
  import type { Act, Item, State } from "../types";
  import Chip from "./Chip.svelte";

  let {
    item,
    people,
    run,
    open,
  }: {
    item: Item;
    people: string[];
    run: (change: Act, message?: string) => Promise<boolean>;
    open: (resource: string) => void;
  } = $props();

  const states: { id: State; label: () => string }[] = [
    { id: "idea", label: () => t("Idea") },
    { id: "focus", label: () => t("Focus") },
    { id: "intermezzo", label: () => t("In between") },
    { id: "delegated", label: () => t("Delegated") },
    { id: "done", label: () => t("Done") },
  ];

  // --- title ---
  let title = $state("");
  let titleFocused = $state(false);
  $effect(() => {
    if (!titleFocused) title = item.title;
  });
  async function saveTitle() {
    titleFocused = false;
    if (title.trim() !== item.title.trim() && title.trim()) await run({ kind: "rename", id: item.id, title });
    else title = item.title;
  }

  // --- status: delegating needs a name ---
  let delegating = $state(false);
  let delegateTo = $state("");
  const changeState = (to: State) => {
    if (to === item.state) return;
    if (to === "delegated") {
      delegating = true;
      delegateTo = "";
      return;
    }
    delegating = false;
    if (to === "done") void run({ kind: "finish", id: item.id }, t("Marked as done"));
    else void run({ kind: "send_to", id: item.id, to });
  };
  async function delegate() {
    const to = delegateTo.trim();
    if (!to) return;
    if (await run({ kind: "delegate", id: item.id, to }, t("Delegated to @{name}", { name: to.replace(/^@/, "") }))) {
      delegating = false;
      delegateTo = "";
    }
  }

  // --- date: typed, no native picker ---
  let day = $state("");
  let dayFocused = $state(false);
  let dayBad = $state(false);
  $effect(() => {
    if (!dayFocused) day = item.date ?? "";
  });
  async function saveDay(text = day) {
    dayFocused = false;
    const parsed = parseDay(text);
    dayBad = parsed === undefined;
    if (parsed === undefined || parsed === item.date) {
      if (!dayBad) day = item.date ?? "";
      return;
    }
    await run({ kind: "set_date", id: item.id, date: parsed });
  }

  // --- resources and steps ---
  let resourceText = $state("");
  let stepText = $state("");
  async function addResource() {
    const target = resourceText.trim();
    if (target && (await run({ kind: "add_resource", id: item.id, target }))) resourceText = "";
  }
  async function addStep() {
    const text = stepText.trim();
    if (text && (await run({ kind: "add_check", id: item.id, text }))) stepText = "";
  }
  let stepEdit = $state<Record<string, string>>({});
  async function saveStep(id: string, original: string) {
    const text = stepEdit[id];
    delete stepEdit[id];
    if (text === undefined || text.trim() === original) return;
    if (!text.trim()) return; // blank: the step stays; removing is its own button
    await run({ kind: "rename_check", id: item.id, check: id, text });
  }

  // --- note: saves while typing ---
  let note = $state("");
  let noteField = $state<HTMLTextAreaElement>();
  let noteTimer: ReturnType<typeof setTimeout> | undefined;
  let noteDirty = false;
  $effect(() => {
    if (document.activeElement !== noteField && !noteDirty) note = item.note;
  });
  function onNoteInput() {
    noteDirty = true;
    clearTimeout(noteTimer);
    noteTimer = setTimeout(flushNote, 400);
  }
  function flushNote() {
    clearTimeout(noteTimer);
    if (!noteDirty) return;
    noteDirty = false;
    void run({ kind: "set_note", id: item.id, note });
  }
  onDestroy(() => {
    flushNote();
  });

  /** Esc closes what is open in the editor first, then the view (the app handles that). */
  function stop(e: KeyboardEvent, close: () => void) {
    if (e.key !== "Escape") return;
    e.stopPropagation();
    close();
  }
  const onEnter = (e: KeyboardEvent, job: () => void) => {
    if (e.key === "Enter" && !e.isComposing) {
      e.preventDefault();
      job();
    }
  };
</script>

<input
  class="title"
  bind:value={title}
  placeholder={t("Title")}
  aria-label={t("Title")}
  onfocus={() => (titleFocused = true)}
  onblur={saveTitle}
  onkeydown={(e) => {
    onEnter(e, () => e.currentTarget instanceof HTMLInputElement && e.currentTarget.blur());
    stop(e, () => {
      title = item.title;
      (e.currentTarget as HTMLInputElement).blur();
    });
  }}
/>

<div class="row" role="group" aria-label={t("Status")}>
  <span class="k">{t("Status")}</span>
  <div class="seg">
    {#each states as s (s.id)}
      <button class:on={item.state === s.id} aria-pressed={item.state === s.id} onclick={() => changeState(s.id)}>
        {s.label()}
      </button>
    {/each}
  </div>
</div>

{#if delegating || item.state === "delegated"}
  <div class="row">
    <span class="k">{t("To")}</span>
    {#if delegating}
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="field"
        autofocus
        list="tq-elab-people"
        bind:value={delegateTo}
        placeholder={t("Delegate to @Name …")}
        aria-label={t("Delegate to @Name …")}
        onkeydown={(e) => {
          onEnter(e, delegate);
          stop(e, () => (delegating = false));
        }}
      />
      <datalist id="tq-elab-people">
        {#each people as name (name)}<option value={name}></option>{/each}
      </datalist>
      <button class="btn" onclick={delegate}>{t("Delegate")}</button>
    {:else}
      <b>@{item.delegated_to}</b>
      <button class="btn" onclick={() => ((delegating = true), (delegateTo = item.delegated_to ?? ""))}>
        {t("Change")}
      </button>
    {/if}
  </div>
{/if}

<div class="row">
  <span class="k">{t("Date")}</span>
  <input
    class="field day"
    class:bad={dayBad}
    bind:value={day}
    placeholder={t("e.g. 27.11. or tomorrow")}
    aria-label={t("Date")}
    title={t("Only noted, no reminder")}
    onfocus={() => (dayFocused = true)}
    onblur={() => saveDay()}
    onkeydown={(e) => {
      onEnter(e, () => saveDay());
      stop(e, () => {
        dayFocused = false;
        dayBad = false;
        day = item.date ?? "";
        (e.currentTarget as HTMLInputElement).blur();
      });
    }}
  />
  <button class="btn" onclick={() => saveDay(t("Today"))}>{t("Today")}</button>
  <button class="btn" onclick={() => saveDay(t("Tomorrow"))}>{t("Tomorrow")}</button>
  {#if item.date}
    <span class="shown">{dateLabel(item.date)}</span>
    <button class="ib" title={t("Remove the date")} aria-label={t("Remove the date")} onclick={() => saveDay("")}>
      <X size={14} />
    </button>
  {/if}
</div>

<h4>{t("Resources")}</h4>
<div class="chips">
  {#each item.resources as r (r.id)}
    <span class="chipwrap">
      <Chip resource={r} onopen={() => open(r.id)} />
      <button
        class="ib x"
        title={t("Remove")}
        aria-label={t("Remove")}
        onclick={() => run({ kind: "remove_resource", id: item.id, resource: r.id })}
      >
        <X size={13} />
      </button>
    </span>
  {/each}
</div>
<div class="add">
  <Plus size={14} />
  <input
    bind:value={resourceText}
    placeholder={t("Add link or file …")}
    aria-label={t("Add link or file …")}
    onkeydown={(e) => onEnter(e, addResource)}
  />
</div>

<h4>{t("Checklist")}</h4>
<ul class="steps" role="list">
  {#each item.checklist as step (step.id)}
    <li>
      <button
        class="box"
        class:on={step.done}
        role="checkbox"
        aria-checked={step.done}
        aria-label={step.text}
        onclick={() => run({ kind: "toggle_check", id: item.id, check: step.id })}
      >
        {#if step.done}<Check size={12} />{/if}
      </button>
      <input
        class="steptext"
        class:done={step.done}
        value={stepEdit[step.id] ?? step.text}
        aria-label={step.text}
        oninput={(e) => (stepEdit[step.id] = e.currentTarget.value)}
        onblur={() => saveStep(step.id, step.text)}
        onkeydown={(e) => {
          onEnter(e, () => (e.currentTarget as HTMLInputElement).blur());
          stop(e, () => {
            delete stepEdit[step.id];
            (e.currentTarget as HTMLInputElement).blur();
          });
        }}
      />
      <button
        class="ib x"
        title={t("Remove")}
        aria-label={t("Remove")}
        onclick={() => run({ kind: "remove_check", id: item.id, check: step.id })}
      >
        <X size={13} />
      </button>
    </li>
  {/each}
</ul>
<div class="add">
  <Plus size={14} />
  <input
    bind:value={stepText}
    placeholder={t("Add a step …")}
    aria-label={t("Add a step …")}
    onkeydown={(e) => onEnter(e, addStep)}
  />
</div>

<h4>{t("Note")}</h4>
<textarea
  bind:this={noteField}
  bind:value={note}
  oninput={onNoteInput}
  onblur={flushNote}
  rows="8"
  placeholder={t("Interim results, commands, conversation notes …")}
  aria-label={t("Note")}
  onkeydown={(e) => stop(e, () => noteField?.blur())}></textarea>

<style>
  .title {
    width: 100%;
    box-sizing: border-box;
    border: 0;
    border-bottom: 1px solid transparent;
    background: none;
    color: var(--tq-ink);
    font: inherit;
    font-size: 21px;
    font-weight: 600;
    padding: 6px 0;
  }
  .title:focus {
    outline: 0;
    border-color: var(--tq-accent);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin: 8px 0 0;
    font-size: 13px;
  }
  .k {
    width: 52px;
    color: var(--tq-ink-2);
    font-size: 12.5px;
    font-weight: 600;
  }
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
  }
  button,
  .field,
  .add input,
  textarea {
    font: inherit;
    font-size: 13px;
    color: var(--tq-ink);
    background: var(--tq-field);
    border: 1px solid var(--tq-line-strong);
    border-radius: 8px;
    padding: 4px 10px;
  }
  button:hover {
    border-color: var(--tq-accent);
  }
  .seg button {
    border-radius: 0;
    margin-left: -1px;
  }
  .seg button:first-child {
    border-radius: 8px 0 0 8px;
    margin-left: 0;
  }
  .seg button:last-child {
    border-radius: 0 8px 8px 0;
  }
  .seg .on {
    background: var(--tq-accent-soft);
    color: var(--tq-accent);
    border-color: var(--tq-accent);
    position: relative;
    z-index: 1;
    font-weight: 600;
  }
  .field:focus,
  .add input:focus,
  textarea:focus {
    outline: 0;
    border-color: var(--tq-accent);
  }
  .day {
    width: 170px;
  }
  .bad {
    border-color: var(--tq-warn, #d9534f);
  }
  .shown {
    color: var(--tq-ink-2);
  }
  .ib {
    display: inline-grid;
    place-items: center;
    padding: 3px;
    border-color: transparent;
    background: none;
    color: var(--tq-ink-3);
    border-radius: 6px;
  }
  .ib:hover {
    color: var(--tq-accent);
  }
  h4 {
    margin: 20px 0 8px;
    padding-top: 14px;
    border-top: 1px solid var(--tq-line);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--tq-ink-2);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chipwrap {
    display: inline-flex;
    align-items: center;
  }
  .add {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 8px;
    color: var(--tq-ink-3);
  }
  .add input {
    flex: 1;
    min-width: 0;
    border-style: dashed;
  }
  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .steps li {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .box {
    flex: none;
    display: inline-grid;
    place-items: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border-radius: 5px;
  }
  .box.on {
    background: var(--tq-accent);
    border-color: var(--tq-accent);
    color: var(--tq-bg);
  }
  .steptext {
    flex: 1;
    min-width: 0;
    border-color: transparent;
    background: none;
  }
  .steptext:hover {
    border-color: var(--tq-line);
  }
  .steptext.done {
    color: var(--tq-ink-3);
    text-decoration: line-through;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    resize: vertical;
    line-height: 1.45;
    padding: 8px 10px;
  }
</style>
