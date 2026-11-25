<script lang="ts">
  // Quick entry (docs/KONZEPT.md 3.1): focused when the popover opens. Shortcuts are recognised live
  // by the core and shown below the field; the legend lists the four keys from urgent to later.
  import { Calendar, File, Forward, Link, Mail, Plus } from "@lucide/svelte";
  import { capture, parseQuick, undo } from "../backend";
  import { dateLabel } from "../format";
  import { t } from "../i18n";
  import { showToast } from "../toast.svelte";
  import type { CaptureKey, Captured, QuickEntry } from "../types";

  let { ondown }: { ondown?: () => void } = $props();

  let input: HTMLInputElement;
  let text = $state("");
  let entry = $state<QuickEntry | null>(null);
  let asked = 0;

  /** Put the cursor into the field. */
  export function focus() {
    input?.focus();
  }

  /** Replace the text (pasting with no target, docs/KONZEPT.md 3.2) and focus the field. */
  export function setText(value: string) {
    text = value;
    void recognise();
    focus();
  }

  async function recognise() {
    const mine = ++asked;
    const result = text.trim() ? await parseQuick(text) : null;
    if (mine === asked) entry = result;
  }

  function message(c: Captured): string {
    switch (c.placed) {
      case "now":
        return c.replaced ? t("Now in progress, the previous one is #2") : t("Now in progress");
      case "focus":
        return t("In focus at #{n}", { n: c.position ?? "?" });
      case "intermezzo":
        return t("Put into “In between”");
      case "idea":
        return t("Saved as idea (Ctrl ⏎ = focus)");
      case "delegated":
        return t("Delegated to @{name}", { name: c.delegated_to ?? "" });
    }
  }

  async function save(key: CaptureKey) {
    if (!text.trim()) return;
    try {
      const saved = await capture(text, key);
      text = "";
      entry = null;
      showToast({ text: message(saved), undo: () => void undo() });
    } catch {
      // Nothing recognisable to save: leave the text as it is.
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.isComposing) {
      e.preventDefault();
      void save(e.ctrlKey && e.shiftKey ? "now" : e.ctrlKey ? "focus" : e.altKey ? "intermezzo" : "idea");
    } else if (e.key === "Escape" && text) {
      // Esc closes step by step: first the text, then the popover.
      e.stopPropagation();
      text = "";
      entry = null;
    } else if (e.key === "ArrowDown" && !text) {
      e.preventDefault();
      ondown?.();
    }
  }
</script>

<div class="qi sx-chrome">
  <label class="qbox">
    <Plus size={16} />
    <input
      bind:this={input}
      bind:value={text}
      oninput={recognise}
      {onkeydown}
      autocomplete="off"
      spellcheck="false"
      aria-label={t("Quick entry")}
      placeholder={t("Quick entry …")}
    />
  </label>
  <div class="qhint">
    {#if entry && (entry.date || entry.delegate_to || entry.resources.length)}
      <span class="rec">
        {t("recognised:")}
        {#if entry.date}<span class="dt"><Calendar size={13} />{dateLabel(entry.date)}</span>{/if}
        {#if entry.delegate_to}<span class="dt"><Forward size={13} />@{entry.delegate_to}</span>{/if}
        {#each entry.resources as r (r.target)}
          <span class="dt" title={r.target}>
            {#if r.type === "url"}<Link size={13} />{:else if r.type === "mail"}<Mail size={13} />{:else}<File
                size={13}
              />{/if}{r.shown}
          </span>
        {/each}
      </span>
    {/if}
    {#if entry?.delegate_to}
      <span><kbd>⏎</kbd> {t("files it under Delegated")}</span>
    {:else}
      <span class="leg" title={t("From urgent to later")}>
        <span class="lg l1" title={t("Right away on #1 and in progress. The current task moves to #2.")}>
          <kbd>{t("Ctrl")} ⇧ ⏎</kbd>{t("Now")}
        </span><i>›</i>
        <span class="lg" title={t("At the end of the focus list. What you are working on stays.")}>
          <kbd>{t("Ctrl")} ⏎</kbd>{t("Focus")}
        </span><i>›</i>
        <span class="lg" title={t("Into the bar for small things in between")}>
          <kbd>Alt ⏎</kbd>{t("In between")}
        </span><i>›</i>
        <span class="lg" title={t("Into the ideas, without urgency")}><kbd>⏎</kbd>{t("Idea")}</span>
      </span>
    {/if}
  </div>
</div>

<style>
  .qi {
    padding: 12px 14px 10px;
    border-bottom: 1px solid var(--sx-chrome-divider);
  }
  .qbox {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--sx-surface);
    border: 1px solid var(--sx-line-strong);
    border-radius: var(--sx-radius-l);
    padding: 0 11px;
    color: var(--sx-ink-2);
  }
  .qbox:focus-within {
    border-color: var(--sx-accent);
    box-shadow: 0 0 0 3px var(--sx-accent-soft);
  }
  input {
    flex: 1;
    border: 0;
    background: none;
    outline: 0;
    padding: 10px 0;
    font-size: 15px;
    color: var(--sx-ink);
  }
  input::placeholder {
    color: var(--sx-ink-3);
  }
  .qhint {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 10px;
    margin-top: 8px;
    font-size: 12px;
    color: var(--sx-ink-2);
    align-items: center;
  }
  .rec {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    width: 100%;
    align-items: center;
  }
  .dt {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--sx-ink-2);
    border: 1px solid var(--sx-line);
    border-radius: var(--sx-radius-l);
    padding: 2px 8px;
    white-space: nowrap;
  }
  .leg {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .leg i {
    font-style: normal;
    color: var(--sx-ink-3);
  }
  .lg {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }
  .lg.l1 {
    color: var(--sx-accent-ink);
    font-weight: 600;
  }
</style>
