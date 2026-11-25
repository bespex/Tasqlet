<script lang="ts">
  // The popover: tabs, quick entry (1.6), notices, the work card (1.7), the focus list and the feedback
  // line. The keyboard in the queue (1.8) follows.
  import { onMount } from "svelte";
  import {
    act,
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
  import { MANY_IN_FOCUS } from "./lib/queue";
  import { showToast } from "./lib/toast.svelte";
  import type { Act, Item, Snapshot } from "./lib/types";
  import Notice from "./lib/ui/Notice.svelte";
  import QuickEntry from "./lib/ui/QuickEntry.svelte";
  import Queue from "./lib/ui/Queue.svelte";
  import Tabs from "./lib/ui/Tabs.svelte";
  import Toast from "./lib/ui/Toast.svelte";
  import WorkCard from "./lib/ui/WorkCard.svelte";

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
  let queue = $state<Queue>();

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

  // Pasting with no text field focused: onto the selected line of the queue a link, file or mail becomes
  // its resource; otherwise the text goes into the quick entry (docs/KONZEPT.md 3.2, 6.6).
  async function onPaste(e: ClipboardEvent) {
    if (typing(e.target)) return;
    const text = e.clipboardData?.getData("text")?.trim();
    if (!text) return;
    e.preventDefault();
    const id = queue?.target();
    if (id) {
      try {
        await act({ kind: "add_resource", id, target: text });
        showToast({ text: t("Added as resource"), undo: () => void undoLast() });
        return;
      } catch {
        // No link, file or mail: fall through to the quick entry.
      }
    }
    quick.setText(text);
  }

  /** Carries out a change from the work card; with a message it is shown, with "Undo". */
  async function run(change: Act, message?: string): Promise<boolean> {
    try {
      await act(change);
    } catch (e) {
      showToast({ text: failure(change, e) });
      return false;
    }
    if (message) showToast({ text: message, undo: () => void undoLast() });
    return true;
  }

  /** What went wrong, in words for the user. */
  function failure(change: Act, e: unknown): string {
    if (change.kind === "add_resource") return t("That is no link, file or mail");
    if (change.kind === "delegate") return t("Delegating needs a name");
    return t("Not possible: {error}", { error: String(e) });
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
    <QuickEntry bind:this={quick} ondown={() => queue?.enter()} />

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
        <WorkCard item={focus[0]} onbreak={snapshot?.on_break ?? false} {run} />
        <Queue bind:this={queue} items={focus.slice(1)} {run} onup={() => quick.focus()} />
        {#if focus.length >= MANY_IN_FOCUS}
          <p class="quiet">{t("Is all of this urgent?")}</p>
        {/if}
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
  .quiet {
    margin: 12px 16px;
    font-size: 12.5px;
    color: var(--tq-ink-3);
    text-align: center;
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
