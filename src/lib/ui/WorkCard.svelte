<script lang="ts">
  // The work card (#1), draft K2 (D20, docs/KONZEPT.md 6.1): no header row, the title on top, a round
  // button at the top right (✓ done, ▶ when "ready"), small round tools below that show on hover or
  // keyboard focus, then resources, checklist and note, and a footer bar with status and date.
  import { onDestroy, tick } from "svelte";
  import { ArrowRight, Check, Coffee, Ellipsis, Maximize2, Play } from "@lucide/svelte";
  import { dateLabel } from "../format";
  import { t } from "../i18n";
  import type { Act, Item } from "../types";
  import Chip from "./Chip.svelte";
  import Section from "./Section.svelte";

  let {
    item,
    onbreak = false,
    run,
  }: {
    item: Item;
    onbreak?: boolean;
    /** Carries out a change; `message` is the feedback line (with "Undo") on success. Resolves to success. */
    run: (change: Act, message?: string) => Promise<boolean>;
  } = $props();

  /** The note grows with its text up to five lines (20 px each, plus padding and border), then scrolls. */
  const NOTE_MAX_PX = 5 * 20 + 12 + 2;

  const ready = $derived(!item.active);
  const steps = $derived(item.checklist.filter((c) => c.done).length);

  // --- title: a double click renames ---
  let renaming = $state(false);
  let titleDraft = $state("");
  async function startRename() {
    titleDraft = item.title;
    renaming = true;
    await tick();
    titleInput?.select();
  }
  let titleInput = $state<HTMLInputElement>();
  async function endRename(save: boolean) {
    if (!renaming) return;
    renaming = false;
    if (save && titleDraft.trim() !== item.title) await run({ kind: "rename", id: item.id, title: titleDraft });
  }

  // --- delegate and the "…" menu ---
  let delegating = $state(false);
  let delegateTo = $state("");
  let delegateInput = $state<HTMLInputElement>();
  let menu = $state(false);
  async function openDelegate() {
    menu = false;
    delegating = true;
    await tick();
    delegateInput?.focus();
  }
  async function delegate() {
    const to = delegateTo.trim();
    if (!to) return;
    const name = to.replace(/^@/, "");
    if (await run({ kind: "delegate", id: item.id, to }, t("Delegated to @{name}", { name }))) {
      delegating = false;
      delegateTo = "";
    }
  }
  /** Escape closes what is open on the card, and only that: the popover stays. */
  function closeOnEscape(e: KeyboardEvent, close: () => void) {
    if (e.key !== "Escape") return;
    e.stopPropagation();
    close();
  }

  // --- adding resources and steps (a "+" under the label opens the entry) ---
  let addingResource = $state(false);
  let resourceText = $state("");
  let resourceInput = $state<HTMLInputElement>();
  let addingStep = $state(false);
  let stepText = $state("");
  let stepInput = $state<HTMLInputElement>();
  async function openResource() {
    addingResource = true;
    await tick();
    resourceInput?.focus();
  }
  async function openStep() {
    addingStep = true;
    await tick();
    stepInput?.focus();
  }
  async function addResource() {
    const target = resourceText.trim();
    if (target && (await run({ kind: "add_resource", id: item.id, target }))) resourceText = "";
  }
  async function addStep() {
    const text = stepText.trim();
    if (text && (await run({ kind: "add_check", id: item.id, text }))) stepText = "";
  }

  // --- note: grows with its content, saves while typing ---
  let note = $state("");
  let noteField = $state<HTMLTextAreaElement>();
  let noteTimer: ReturnType<typeof setTimeout> | undefined;
  let noteDirty = false;
  let noteFor = "";
  $effect(() => {
    // Take over the note from outside, but never while the user is typing in it.
    if (item.id !== noteFor) {
      flushNote();
      noteFor = item.id;
    }
    if (document.activeElement !== noteField && !noteDirty) note = item.note;
  });
  function fitNote() {
    if (!noteField) return;
    noteField.style.height = "auto";
    noteField.style.height = `${Math.min(noteField.scrollHeight + 2, NOTE_MAX_PX)}px`;
  }
  $effect(() => {
    void note;
    void noteField;
    // After the text has reached the field, or its height would be measured on the old text.
    void tick().then(fitNote);
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
    void run({ kind: "set_note", id: noteFor || item.id, note });
  }
  onDestroy(flushNote);

  const go = () =>
    ready ? run({ kind: "start", id: item.id }) : run({ kind: "finish", id: item.id }, t("Marked as done"));
  const finish = () => run({ kind: "finish", id: item.id }, t("Marked as done"));
  async function sendBack(change: Act, message: string) {
    menu = false;
    await run(change, message);
  }
</script>

{#if onbreak}
  <section class="card zone-focus pause" aria-live="polite">
    <Coffee size={22} />
    <p>
      <b>{t("Break.")}</b>
      {t("Then back to")} <span class="then">{item.title}</span>
    </p>
    <button class="primary" onclick={() => run({ kind: "resume" })}><Play size={15} />{t("Continue")}</button>
  </section>
{:else}
  <section class="card zone-focus" class:active={item.active} class:ready>
    <div class="top">
      {#if renaming}
        <input
          class="rename"
          bind:this={titleInput}
          bind:value={titleDraft}
          aria-label={t("Title")}
          onkeydown={(e) => {
            if (e.key === "Enter") void endRename(true);
            closeOnEscape(e, () => void endRename(false));
          }}
          onblur={() => endRename(true)}
        />
      {:else}
        <h2>
          <button class="title" title={t("Double-click to rename")} ondblclick={startRename}>
            {item.title || t("Untitled")}
          </button>
        </h2>
      {/if}
      <button
        class="go"
        class:ready
        title={ready ? t("Start (Enter)") : t("Done")}
        aria-label={ready ? t("Start") : t("Done")}
        onclick={go}
      >
        {#if ready}<Play size={17} fill="currentColor" />{:else}<Check size={18} strokeWidth={2.6} />{/if}
      </button>
    </div>

    <div class="tools" class:open={delegating || menu}>
      {#if delegating}
        <!-- svelte-ignore a11y_autofocus -->
        <input
          class="delegate"
          bind:this={delegateInput}
          bind:value={delegateTo}
          placeholder={t("Delegate to @Name …")}
          aria-label={t("Delegate to")}
          onkeydown={(e) => {
            if (e.key === "Enter") void delegate();
            closeOnEscape(e, () => (delegating = false));
          }}
        />
      {:else}
        {#if ready}
          <button class="rb" title={t("Done")} aria-label={t("Done")} onclick={finish}><Check size={14} /></button>
        {:else}
          <button
            class="rb"
            title={t("Coffee break")}
            aria-label={t("Coffee break")}
            onclick={() => run({ kind: "pause" })}
          >
            <Coffee size={14} />
          </button>
        {/if}
        <button class="rb" title={t("Delegate")} aria-label={t("Delegate")} onclick={openDelegate}>
          <ArrowRight size={14} />
        </button>
        <span class="menuwrap">
          <button
            class="rb"
            title={t("More")}
            aria-label={t("More")}
            aria-haspopup="menu"
            aria-expanded={menu}
            onclick={() => (menu = !menu)}
          >
            <Ellipsis size={14} />
          </button>
          {#if menu}
            <div
              class="menu"
              role="menu"
              tabindex="-1"
              onkeydown={(e) => closeOnEscape(e, () => (menu = false))}
              onmouseleave={() => (menu = false)}
            >
              <button
                role="menuitem"
                onclick={() => sendBack({ kind: "to_back", id: item.id }, t("Back in the queue"))}
              >
                {t("Back to the queue")}
              </button>
              <button
                role="menuitem"
                onclick={() => sendBack({ kind: "send_to", id: item.id, to: "intermezzo" }, t("Put into “In between”"))}
              >
                {t("In between")}
              </button>
              <button
                role="menuitem"
                onclick={() => sendBack({ kind: "send_to", id: item.id, to: "idea" }, t("Saved as idea"))}
              >
                {t("To the ideas")}
              </button>
            </div>
          {/if}
        </span>
      {/if}
    </div>

    <Section label={t("Resources")} onadd={openResource}>
      <div class="chips">
        {#each item.resources.slice(0, 3) as r, i (r.id)}
          <Chip resource={r} digit={i + 1} />
        {/each}
        {#if item.resources.length > 3}<Chip />{/if}
        {#if !item.resources.length && !addingResource}<span class="none">{t("None yet")}</span>{/if}
      </div>
      {#if addingResource}
        <input
          class="add"
          bind:this={resourceInput}
          bind:value={resourceText}
          placeholder={t("Link, file or mail …")}
          aria-label={t("Add resource")}
          onkeydown={(e) => {
            if (e.key === "Enter") void addResource();
            closeOnEscape(e, () => (addingResource = false));
          }}
        />
      {/if}
    </Section>

    <Section label={t("Checklist")} onadd={openStep}>
      {#if item.checklist.length}
        <ul class="steps">
          {#each item.checklist as step (step.id)}
            <li>
              <label class:done={step.done}>
                <input
                  type="checkbox"
                  checked={step.done}
                  onchange={() => run({ kind: "toggle_check", id: item.id, check: step.id })}
                />
                <span>{step.text}</span>
              </label>
            </li>
          {/each}
        </ul>
      {:else if !addingStep}
        <span class="none">{t("No steps yet")}</span>
      {/if}
      {#if addingStep}
        <input
          class="add"
          bind:this={stepInput}
          bind:value={stepText}
          placeholder={t("New step …")}
          aria-label={t("Add step")}
          onkeydown={(e) => {
            if (e.key === "Enter") void addStep();
            closeOnEscape(e, () => (addingStep = false));
          }}
        />
      {/if}
    </Section>

    <Section label={t("Note")}>
      <textarea
        class="note"
        rows="1"
        bind:this={noteField}
        bind:value={note}
        placeholder={t("Keywords, results …")}
        aria-label={t("Note")}
        oninput={onNoteInput}
        onblur={flushNote}
        onkeydown={(e) => closeOnEscape(e, () => noteField?.blur())}></textarea>
    </Section>

    <footer class="bar">
      <span class="status" class:ready>{ready ? t("Ready") : t("In progress")}</span>
      {#if item.checklist.length}<span>{steps}/{item.checklist.length}</span>{/if}
      {#if item.date}<span>{dateLabel(item.date)}</span>{/if}
      <span class="sp"></span>
      <button class="elab" disabled title={t("Comes with the elaboration view")}>
        <Maximize2 size={13} />{t("Elaborate")}
      </button>
    </footer>
  </section>
{/if}

<style>
  .card {
    position: relative;
    margin: 14px;
    padding: 12px 14px 0;
    background: var(--tq-card);
    border: var(--tq-f-border-w) solid var(--tq-line);
    border-radius: 12px;
    box-shadow: var(--tq-f-shadow);
  }
  .card.active {
    border-color: var(--tq-f-border);
  }
  .top {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  h2 {
    flex: 1;
    margin: 0;
    font-size: 17px;
    line-height: 1.3;
    font-weight: 600;
    min-width: 0;
  }
  .title {
    border: 0;
    background: none;
    padding: 0;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: default;
  }
  .rename {
    flex: 1;
    min-width: 0;
    font: inherit;
    font-size: 17px;
    font-weight: 600;
    background: var(--tq-field);
    color: var(--tq-ink);
    border: 1px solid var(--tq-line-strong);
    border-radius: 6px;
    padding: 2px 6px;
  }
  .go {
    flex: none;
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    border: 2px solid var(--tq-accent);
    background: none;
    color: var(--tq-accent);
    padding: 0;
  }
  .go:hover,
  .go.ready {
    background: var(--tq-accent);
    color: var(--tq-on-accent);
  }
  .go.ready:hover {
    filter: brightness(1.1);
  }
  /* The tools lie over the card, right under the round button, and take no room of their own. They
     are solid and cast a shadow, so they stand out from whatever text or chips lie beneath. */
  .tools {
    position: absolute;
    top: 52px;
    right: 14px;
    z-index: 3;
    display: flex;
    align-items: center;
    gap: 6px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.12s;
  }
  .card:hover .tools,
  .card:has(:focus-visible) .tools,
  .tools.open {
    opacity: 1;
    pointer-events: auto;
  }
  .rb {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    padding: 0;
    border-radius: 50%;
    border: 1px solid var(--tq-line-strong);
    background: var(--tq-card);
    color: var(--tq-ink);
    box-shadow:
      0 0 0 3px var(--tq-card),
      0 3px 10px rgba(0, 0, 0, 0.45);
  }
  .rb:hover {
    background: var(--tq-accent);
    border-color: var(--tq-accent);
    color: var(--tq-on-accent);
  }
  .delegate,
  .add {
    width: 100%;
    box-sizing: border-box;
    font: inherit;
    font-size: 13px;
    background: var(--tq-field);
    color: var(--tq-ink);
    border: 1px solid var(--tq-line);
    border-radius: 8px;
    padding: 5px 8px;
  }
  .delegate {
    width: 250px;
    background: var(--tq-card);
    border-color: var(--tq-line-strong);
    box-shadow:
      0 0 0 3px var(--tq-card),
      0 3px 10px rgba(0, 0, 0, 0.45);
  }
  .delegate:focus,
  .add:focus,
  .note:focus {
    outline: 0;
    border-color: var(--tq-accent);
  }
  .add {
    margin-top: 8px;
  }
  .menuwrap {
    position: relative;
  }
  .menu {
    position: absolute;
    right: 0;
    top: 30px;
    z-index: 5;
    display: grid;
    min-width: 170px;
    padding: 4px;
    background: var(--tq-card);
    border: 1px solid var(--tq-line-strong);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }
  .menu button {
    border: 0;
    background: none;
    text-align: left;
    padding: 6px 10px;
    border-radius: 6px;
    color: var(--tq-ink);
    font-size: 13px;
  }
  .menu button:hover {
    background: var(--tq-hover);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .none {
    color: var(--tq-ink-3);
    font-size: 13px;
  }
  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .steps label {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    font-size: 14px;
  }
  .steps input {
    margin-top: 3px;
    accent-color: var(--tq-accent);
  }
  .steps .done span {
    color: var(--tq-ink-3);
    text-decoration: line-through;
  }
  .note {
    width: 100%;
    box-sizing: border-box;
    resize: none;
    font: inherit;
    font-size: 14px;
    line-height: 20px;
    background: var(--tq-field);
    color: var(--tq-ink);
    border: 1px solid var(--tq-line);
    border-radius: 8px;
    padding: 6px 8px;
    overflow-y: auto;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 10px -14px 0;
    padding: 7px 14px;
    border-top: 1px solid var(--tq-line);
    font-size: 12px;
    color: var(--tq-ink-2);
  }
  .status {
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 11px;
  }
  .status.ready {
    color: var(--tq-accent);
  }
  .sp {
    flex: 1;
  }
  .elab {
    display: inline-flex;
    gap: 5px;
    align-items: center;
    border: 0;
    background: none;
    color: var(--tq-ink-2);
    font-size: 12px;
  }
  .elab:disabled {
    opacity: 0.55;
  }
  .pause {
    display: grid;
    justify-items: center;
    gap: 10px;
    padding: 22px 16px;
    text-align: center;
    color: var(--tq-ink-2);
  }
  .pause p {
    margin: 0;
  }
  .then {
    color: var(--tq-ink);
    font-weight: 600;
  }
  .primary {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    border: 0;
    border-radius: 8px;
    padding: 7px 14px;
    background: var(--tq-accent);
    color: var(--tq-on-accent);
    font-weight: 600;
  }
</style>
