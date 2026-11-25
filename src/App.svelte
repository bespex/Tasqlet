<script lang="ts">
  // The popover: tabs, quick entry (1.6), notices, the focus list and the feedback line. The work card
  // (1.7) and the keyboard in the queue (1.8) follow.
  import { onMount } from "svelte";
  import { Play } from "@lucide/svelte";
  import {
    hidePopover,
    inTauri,
    loadSnapshot,
    restoreBackup,
    undo,
    watchCapture,
    watchChanged,
    watchShown,
  } from "./lib/backend";
  import { t } from "./lib/i18n";
  import { showToast } from "./lib/toast.svelte";
  import type { Item, Snapshot } from "./lib/types";
  import Chip from "./lib/ui/Chip.svelte";
  import Notice from "./lib/ui/Notice.svelte";
  import QuickEntry from "./lib/ui/QuickEntry.svelte";
  import Row from "./lib/ui/Row.svelte";
  import Tabs from "./lib/ui/Tabs.svelte";
  import Toast from "./lib/ui/Toast.svelte";

  let snapshot = $state<Snapshot | null>(null);
  let view = $state("focus");
  const preview = !inTauri();

  const items = $derived(snapshot?.items ?? []);
  const focus = $derived(items.filter((i) => i.state === "focus").sort((a, b) => a.order - b.order));
  const count = (state: Item["state"]) => items.filter((i) => i.state === state).length;
  const tabs = $derived([
    { id: "focus", label: t("Focus"), count: count("focus") },
    { id: "idea", label: t("Ideas"), count: count("idea") },
    { id: "delegated", label: t("Delegated"), count: count("delegated") },
  ]);

  let quick: QuickEntry;
  let queue = $state<HTMLElement>();

  const reload = () => loadSnapshot().then((s) => (snapshot = s));

  onMount(() => {
    void reload();
    quick.focus();
    const shown = () => {
      void reload();
      quick.focus();
    };
    const stops = [watchShown(shown), watchChanged(reload), watchCapture(() => quick.focus())];
    return () => stops.forEach((stop) => void stop.then((f) => f()));
  });

  /** Whether a text field has the focus: then keys and pasting belong to it. */
  const typing = (target: EventTarget | null) =>
    target instanceof HTMLElement && (target.isContentEditable || ["INPUT", "TEXTAREA"].includes(target.tagName));

  async function undoLast() {
    try {
      await undo();
      showToast({ text: t("Undone") });
    } catch {
      showToast({ text: t("Nothing to undo") });
    }
  }

  // Pasting with no text field focused and no item as target puts the text into the quick entry
  // (docs/KONZEPT.md 3.2). Pasting onto a hovered or selected item follows with plan 1.9.
  function onPaste(e: ClipboardEvent) {
    if (typing(e.target)) return;
    const text = e.clipboardData?.getData("text")?.trim();
    if (!text) return;
    e.preventDefault();
    quick.setText(text);
  }

  async function restore() {
    try {
      await restoreBackup();
      showToast({ text: t("Restored the previous version") });
      await reload();
    } catch (e) {
      showToast({ text: t("Restoring failed: {error}", { error: String(e) }) });
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") void hidePopover();
    else if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === "z" && !typing(e.target)) {
      e.preventDefault();
      void undoLast();
    }
  }
</script>

<svelte:window onkeydown={onKey} onpaste={onPaste} />

<div class="stage" class:preview>
  <div class="pop">
    <Tabs {tabs} active={view} onselect={(id) => (view = id)} />
    <QuickEntry bind:this={quick} ondown={() => queue?.querySelector<HTMLElement>("[role=option]")?.focus()} />

    {#if snapshot?.tray_missing}
      <Notice
        title={t("No tray found")}
        text={t(
          "Tasqlet lives in the tray, but this desktop shows no tray icons. On GNOME, install the “AppIndicator and KStatusNotifierItem Support” extension; until then Tasqlet opens as a window.",
        )}
      />
    {/if}
    {#if snapshot?.load_error}
      <Notice
        warn
        title={t("The data could not be loaded")}
        text={`${snapshot.load_error.message} ${t("Tasqlet is read-only and changes nothing on disk.")}`}
        action={snapshot.load_error.backup ? t("Restore the previous version") : undefined}
        onaction={restore}
      />
    {/if}

    <div class="body">
      {#if focus.length === 0}
        <div class="empty">
          <p><b>{t("Nothing urgent.")}</b><br />{t("Ctrl ⏎ in the quick entry puts a task into focus.")}</p>
        </div>
      {:else}
        {@const top = focus[0]}
        <section class="card zone-focus" class:active={top.active}>
          <h2>{top.title}</h2>
          {#if top.resources.length}
            <div class="chips">
              {#each top.resources.slice(0, 3) as r, i (r.id)}
                <Chip resource={r} digit={i + 1} />
              {/each}
              {#if top.resources.length > 3}<Chip />{/if}
            </div>
          {/if}
        </section>
        <div role="listbox" aria-label={t("Focus")} bind:this={queue}>
          {#each focus.slice(1) as item, i (item.id)}
            <Row num={i + 2} title={item.title}>
              {#snippet indicators()}
                {#if item.last_active}<span class="badge">{t("↩ last worked on")}</span>{/if}
              {/snippet}
              {#snippet tools()}
                <button class="ib run" title={t("Take over now (Enter)")}><Play size={16} /></button>
              {/snippet}
            </Row>
          {/each}
        </div>
      {/if}
    </div>

    <footer class="zone-chrome">
      {preview ? t("Browser preview with sample data") : t("Esc hides the popover")}
    </footer>
    <Toast />
  </div>
</div>

<style>
  .stage {
    height: 100vh;
  }
  .stage.preview {
    display: grid;
    place-items: center;
    background: radial-gradient(110% 80% at 18% 8%, #55697a 0, #2e3b47 45%, #1a2128 100%);
  }
  .pop {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background: var(--tq-bg);
    color: var(--tq-ink);
  }
  .preview .pop {
    width: 470px;
    height: 680px;
    border: 1px solid var(--tq-line);
    border-radius: 14px;
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.38);
  }
  .body {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .card {
    margin: 14px;
    padding: 14px 16px 16px;
    background: var(--tq-card);
    border: var(--tq-f-border-w) solid var(--tq-line);
    border-radius: 12px;
    box-shadow: var(--tq-f-shadow);
  }
  .card.active {
    border-color: var(--tq-f-border);
  }
  h2 {
    font-size: 17px;
    line-height: 1.3;
    margin: 0;
    font-weight: 600;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 12px;
  }
  .badge {
    white-space: nowrap;
  }
  .ib {
    width: 30px;
    height: 30px;
    display: inline-grid;
    place-items: center;
    border: 1px solid transparent;
    background: none;
    border-radius: 8px;
    color: var(--tq-ink-2);
  }
  .ib:hover {
    background: var(--tq-hover);
    color: var(--tq-ink);
  }
  .ib.run {
    color: var(--tq-accent);
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
  footer {
    padding: 10px 14px;
    font-size: 12px;
    border-top: 1px solid var(--tq-c-divider);
    color: var(--tq-ink-2);
  }
</style>
