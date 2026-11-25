<script lang="ts">
  // The work card (#1), draft K2 (D20, docs/KONZEPT.md 6.1): no header row, the title on top, a round
  // button at the top right (✓ done, ▶ when "ready"), small round tools below that show on hover or
  // keyboard focus, then resources, checklist and note, and a footer bar with status and date.
  import { onDestroy, tick } from "svelte";
  import { ArrowRight, Check, Coffee, Ellipsis, File, Link, Mail, Maximize2, Play, Plus } from "@lucide/svelte";
  import { dateLabel } from "../format";
  import { t } from "../i18n";
  import type { Act, Item } from "../types";
  import Chip from "./Chip.svelte";
  import Section from "./Section.svelte";
  import { keyLabel, withKey } from "../keys";

  let {
    item,
    onbreak = false,
    run,
    open,
    openAll,
    people = [],
    onelaborate,
  }: {
    item: Item;
    onbreak?: boolean;
    /** Opens one resource (ranks it first). */
    open: (resource: string) => void;
    /** Opens all resources, most important first. */
    openAll: () => void;
    /** Names of earlier recipients, offered when delegating. */
    people?: string[];
    /** Opens the elaboration of this task. */
    onelaborate: () => void;
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
  let resMenu = $state(false);
  let addingResource = $state(false);
  let resourceText = $state("");
  let resourceInput = $state<HTMLInputElement>();
  let addingStep = $state(false);
  let stepText = $state("");
  let stepInput = $state<HTMLInputElement>();
  async function openAdd() {
    resMenu = false;
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
  <section class="card sx-emphasis pause" aria-live="polite" data-drop={item.id}>
    <Coffee size={22} />
    <p>
      <b>{t("Break.")}</b>
      {t("Then back to")} <span class="then">{item.title}</span>
    </p>
    <button class="primary" title={withKey(t("Continue"), "Ctrl+P")} onclick={() => run({ kind: "resume" })}>
      <Play size={15} />{t("Continue")}
    </button>
  </section>
{:else}
  <section class="card sx-emphasis" class:active={item.active} class:ready data-drop={item.id}>
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

    <div class="tools" class:open={delegating || menu} class:wide={delegating}>
      {#if delegating}
        <datalist id="tq-people">
          {#each people as name (name)}<option value={name}></option>{/each}
        </datalist>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          class="delegate"
          bind:this={delegateInput}
          bind:value={delegateTo}
          list="tq-people"
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
            title={withKey(t("Coffee break"), "Ctrl+P")}
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

    <Section label={t("Resources")} onadd={openAdd} empty={!item.resources.length && !addingResource}>
      <div class="chips">
        {#each item.resources.slice(0, 3) as r (r.id)}
          <Chip
            resource={r}
            digit={r.digit}
            onopen={() => open(r.id)}
            onremove={() => run({ kind: "remove_resource", id: item.id, resource: r.id }, t("Resource removed"))}
          />
        {/each}
        {#if item.resources.length > 3}
          <span class="menuwrap">
            <Chip onopen={() => (resMenu = !resMenu)} />
            {#if resMenu}
              <div
                class="menu resmenu"
                role="menu"
                tabindex="-1"
                onkeydown={(e) => closeOnEscape(e, () => (resMenu = false))}
                onmouseleave={() => (resMenu = false)}
              >
                {#each item.resources as r (r.id)}
                  <button
                    role="menuitem"
                    title={r.target}
                    onclick={() => {
                      resMenu = false;
                      open(r.id);
                    }}
                  >
                    {#if r.type === "url"}<Link size={14} />{:else if r.type === "mail"}<Mail size={14} />{:else}<File
                        size={14}
                      />{/if}
                    <span class="rl">{r.shown ?? r.label ?? r.target}</span>
                    {#if r.digit}<span class="rk">{r.digit}</span>{/if}
                  </button>
                {/each}
                <hr />
                <button
                  role="menuitem"
                  onclick={() => {
                    resMenu = false;
                    openAll();
                  }}
                >
                  {t("Open all ({n})", { n: item.resources.length })}<span class="rk">{keyLabel("Ctrl+O")}</span>
                </button>
                <button role="menuitem" onclick={openAdd}><Plus size={14} />{t("Add link or file …")}</button>
              </div>
            {/if}
          </span>
        {/if}
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

    <Section label={t("Checklist")} onadd={openStep} empty={!item.checklist.length && !addingStep}>
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
      <button class="elab" title={withKey(t("Elaborate"), "Ctrl+E")} onclick={onelaborate}>
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
    background: var(--sx-surface);
    border: var(--sx-emph-border-w) solid var(--sx-line);
    border-radius: var(--sx-radius-l);
    box-shadow: var(--sx-emph-shadow);
  }
  /* The card is never lower than its tools: they start 54 px from the top and are three 28 px
     buttons with two 9 px gaps (156 px), plus room for halo and glow below. */
  .card:not(.pause) {
    min-height: 172px;
  }
  .card.active {
    border-color: var(--sx-emph-border);
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
    background: var(--sx-field);
    color: var(--sx-ink);
    border: 1px solid var(--sx-line-strong);
    border-radius: var(--sx-radius-m);
    padding: 2px 6px;
  }
  .go {
    flex: none;
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    border: 2px solid var(--sx-accent);
    background: none;
    color: var(--sx-accent-ink);
    padding: 0;
  }
  .go {
    transition: box-shadow 0.12s;
  }
  .go:hover,
  .go.ready {
    background: var(--sx-accent-fill);
    color: var(--sx-on-accent);
  }
  /* The same soft glow as on the tools, only while hovering. */
  .go:hover {
    box-shadow: 0 0 11px 1px color-mix(in srgb, var(--sx-accent) 50%, transparent);
  }
  .go.ready:hover {
    filter: brightness(1.1);
  }
  /* The tools lie over the card in a column, centred under the round button, and take no room of
     their own. They are solid with a halo and a soft glow, so they stand out from whatever text or
     chips lie beneath. The round button is 34 px wide, the tools 28 px: 3 px more on the right. */
  .tools {
    position: absolute;
    top: 54px;
    right: 17px;
    z-index: 3;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 9px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.12s;
  }
  .tools.wide {
    right: 14px;
    align-items: flex-end;
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
    border: 1px solid var(--sx-accent);
    background: var(--sx-surface);
    color: var(--sx-ink);
    box-shadow:
      0 0 0 3px var(--sx-surface),
      0 0 7px 0 color-mix(in srgb, var(--sx-accent) 28%, transparent),
      0 3px 10px rgba(0, 0, 0, 0.4);
  }
  .rb:hover {
    background: var(--sx-accent-fill);
    color: var(--sx-on-accent);
    box-shadow:
      0 0 0 3px var(--sx-surface),
      0 0 11px 1px color-mix(in srgb, var(--sx-accent) 50%, transparent),
      0 3px 10px rgba(0, 0, 0, 0.4);
  }
  .delegate,
  .add {
    width: 100%;
    box-sizing: border-box;
    font: inherit;
    font-size: 13px;
    background: var(--sx-field);
    color: var(--sx-ink);
    border: 1px solid var(--sx-line);
    border-radius: var(--sx-radius-l);
    padding: 5px 8px;
  }
  .delegate {
    width: 250px;
    background: var(--sx-surface);
    border-color: var(--sx-line-strong);
    box-shadow:
      0 0 0 3px var(--sx-surface),
      0 0 7px 0 color-mix(in srgb, var(--sx-accent) 28%, transparent),
      0 3px 10px rgba(0, 0, 0, 0.4);
  }
  .delegate:focus,
  .add:focus,
  .note:focus {
    outline: 0;
    border-color: var(--sx-accent);
  }
  .add {
    margin-top: 8px;
  }
  .menuwrap {
    position: relative;
    display: inline-flex;
  }
  .menu {
    position: absolute;
    right: 38px;
    top: -4px;
    z-index: 5;
    display: grid;
    min-width: 170px;
    padding: 4px;
    background: var(--sx-surface);
    border: 1px solid var(--sx-line-strong);
    border-radius: var(--sx-radius-l);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }
  .menu button {
    border: 0;
    background: none;
    text-align: left;
    padding: 6px 10px;
    border-radius: var(--sx-radius-m);
    color: var(--sx-ink);
    font-size: 13px;
  }
  .resmenu {
    right: 0;
    left: auto;
    top: 34px;
    min-width: 270px;
    max-width: 300px;
  }
  .resmenu button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
  .resmenu :global(svg) {
    flex: none;
    color: var(--sx-ink-2);
  }
  .resmenu .rl {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .resmenu .rk {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--sx-ink-2);
    border: 1px solid var(--sx-line);
    border-radius: var(--sx-radius-s);
    padding: 0 4px;
  }
  .resmenu hr {
    border: 0;
    border-top: 1px solid var(--sx-line);
    margin: 4px 0;
  }
  .menu button:hover {
    background: var(--sx-hover);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
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
    /* Long step text must not squeeze the box (it is a flex item). */
    flex: none;
    margin-top: 3px;
    accent-color: var(--sx-accent);
  }
  .steps .done span {
    color: var(--sx-ink-3);
    text-decoration: line-through;
  }
  .note {
    width: 100%;
    box-sizing: border-box;
    resize: none;
    font: inherit;
    font-size: 14px;
    line-height: 20px;
    background: var(--sx-field);
    color: var(--sx-ink);
    border: 1px solid var(--sx-line);
    border-radius: var(--sx-radius-l);
    padding: 6px 8px;
    overflow-y: auto;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 10px -14px 0;
    padding: 7px 14px;
    border-top: 1px solid var(--sx-line);
    font-size: 12px;
    color: var(--sx-ink-2);
  }
  .status {
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 11px;
  }
  .status.ready {
    color: var(--sx-accent-ink);
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
    color: var(--sx-ink-2);
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
    color: var(--sx-ink-2);
  }
  .pause p {
    margin: 0;
  }
  .then {
    color: var(--sx-ink);
    font-weight: 600;
  }
  .primary {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    border: 0;
    border-radius: var(--sx-radius-l);
    padding: 7px 14px;
    background: var(--sx-accent-fill);
    color: var(--sx-on-accent);
    font-weight: 600;
  }
</style>
