<script lang="ts">
  // The popover: tabs, quick entry (1.6), notices, the work card (1.7), the focus list and the feedback
  // line. The keyboard in the queue (1.8) follows.
  import { onMount, tick } from "svelte";
  import {
    act,
    hidePopover,
    inTauri,
    loadSnapshot,
    openAll,
    openResource,
    restoreBackup,
    undo,
    watchCapture,
    watchChanged,
    watchHidden,
    watchShown,
    watchDrop,
    setPinned,
    setWide,
    type DropEvent,
  } from "./lib/backend";
  import { t } from "./lib/i18n";
  import { MANY_IN_FOCUS } from "./lib/queue";
  import { showToast } from "./lib/toast.svelte";
  import type { Act, Item, Snapshot } from "./lib/types";
  import Notice from "./lib/ui/Notice.svelte";
  import ElaborateView from "./lib/ui/ElaborateView.svelte";
  import DelegatedView from "./lib/ui/DelegatedView.svelte";
  import IdeasView from "./lib/ui/IdeasView.svelte";
  import IntermezzoBar from "./lib/ui/IntermezzoBar.svelte";
  import QuickEntry from "./lib/ui/QuickEntry.svelte";
  import SettingsView from "./lib/ui/SettingsView.svelte";
  import SearchView from "./lib/ui/SearchView.svelte";
  import Queue from "./lib/ui/Queue.svelte";
  import Tabs from "./lib/ui/Tabs.svelte";
  import Toast from "./lib/ui/Toast.svelte";
  import WorkCard from "./lib/ui/WorkCard.svelte";

  let snapshot = $state<Snapshot | null>(null);
  let view = $state("focus");
  let searching = $state(false);
  let settings = $state(false);
  /** Pinned: the popover stays when another window gets the focus (D37). */
  let pinned = $state(false);
  function togglePin() {
    pinned = !pinned;
    void setPinned(pinned);
  }
  /** The item being elaborated (plan 2.4), or null. */
  let elab = $state<string | null>(null);
  const preview = !inTauri();

  const items = $derived(snapshot?.items ?? []);
  const focus = $derived(items.filter((i) => i.state === "focus").sort((a, b) => a.order - b.order));
  const count = (state: Item["state"]) => items.filter((i) => i.state === state).length;
  const tabs = $derived([
    { id: "focus", label: t("Focus"), count: count("focus") },
    { id: "idea", label: t("Ideas"), count: count("idea") },
    { id: "delegated", label: t("Delegated"), count: count("delegated") },
  ]);

  /** Everyone a task was handed to so far, for the suggestions when delegating (plan 2.3). */
  const people = $derived([...new Set(items.map((i) => i.delegated_to).filter((n): n is string => !!n))].sort());

  let quick: QuickEntry;
  let queue = $state<Queue>();

  function openSearch() {
    settings = false;
    searching = true;
  }
  function openElab(id: string | null | undefined) {
    if (!id) return;
    settings = false;
    searching = false;
    elab = id;
  }
  function closeElab() {
    elab = null;
    void tick().then(() => quick.focus());
  }
  // The popover is twice as wide while elaborating.
  const wide = $derived(elab !== null);
  $effect(() => {
    void setWide(wide);
  });
  /** Ctrl+E: the selected or hovered queue line, else the current task, else the newest idea. */
  const elabTarget = () =>
    queue?.target() ??
    (view === "idea"
      ? items.filter((i) => i.state === "idea").sort((a, b) => b.created_at.localeCompare(a.created_at))[0]?.id
      : undefined) ??
    focus[0]?.id;
  function closeSettings() {
    settings = false;
    void tick().then(() => quick.focus());
  }
  function closeSearch() {
    searching = false;
    void tick().then(() => quick.focus());
  }
  function selectTab(id: string) {
    view = id;
    settings = false;
    elab = null;
    if (searching) closeSearch();
  }

  const reload = () => loadSnapshot().then((s) => (snapshot = s));

  onMount(() => {
    void reload();
    quick.focus();
    const shown = () => {
      void reload();
      quick.focus();
    };
    // Hidden while elaborating or in the settings: the next opening is the normal popover again.
    const hidden = () => {
      elab = null;
      settings = false;
      pinned = false;
    };
    const stops = [
      watchShown(shown),
      watchHidden(hidden),
      watchChanged(reload),
      watchCapture(() => quick.focus()),
      watchDrop(onDrop),
    ];
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
    // A line under the pointer or selected, else the work card under the pointer.
    const id = queue?.target() ?? (document.querySelector(".card:hover") ? focus[0]?.id : undefined);
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

  // Dropping files or links (plan 2.5): onto an entry they become its resources, elsewhere they go
  // into the quick entry (D37). While dragging, the entry under the pointer is outlined.
  let dropOver: Element | null = null;
  function markDrop(el: Element | null) {
    if (el === dropOver) return;
    dropOver?.classList.remove("drop-over");
    el?.classList.add("drop-over");
    dropOver = el;
  }
  const dropTarget = (x: number, y: number) => document.elementFromPoint(x, y)?.closest("[data-drop]") ?? null;
  /** A path with spaces as a `file://` link, as the quick entry reads words. */
  const asWord = (target: string) =>
    target.startsWith("/") && /\s/.test(target)
      ? "file://" + target.split("/").map(encodeURIComponent).join("/")
      : target;
  async function onDrop(e: DropEvent) {
    if (e.type === "leave") return markDrop(null);
    const el = dropTarget(e.x, e.y);
    if (e.type === "over") return markDrop(el);
    markDrop(null);
    const id = el?.getAttribute("data-drop");
    let added = 0;
    if (id) {
      for (const target of e.targets) {
        try {
          await act({ kind: "add_resource", id, target });
          added++;
        } catch {
          // Already there, or no link, file or mail.
        }
      }
    }
    if (added) {
      showToast({
        text: added === 1 ? t("Added as resource") : t("{n} added as resources", { n: added }),
        undo: async () => {
          for (let i = 0; i < added; i++) await undo().catch(() => {});
          showToast({ text: t("Undone") });
        },
      });
    } else if (e.targets.length) {
      settings = false;
      elab = null;
      if (searching) searching = false;
      await tick();
      quick.setText(e.targets.map(asWord).join(" "));
    }
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

  /** Opens a resource (or all of a task's), then leaves the way: the popover hides, the chips re-rank. */
  async function openFor(id: string, resource: string | null) {
    try {
      await (resource ? openResource(id, resource) : openAll(id));
    } catch (e) {
      showToast({ text: t("Could not open: {error}", { error: String(e) }) });
      return;
    }
    void reload();
    void hidePopover();
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
    // Alt+1 … Alt+9: the n-th resource of the current task, whatever has the focus (6.6).
    if (e.altKey && !e.ctrlKey && !e.shiftKey && /^Digit[1-9]$/.test(e.code) && focus[0]) {
      e.preventDefault();
      const digit = Number(e.code.slice(5));
      const found = focus[0].resources.find((r) => r.digit === digit);
      if (found) void openFor(focus[0].id, found.id);
      return;
    }
    const ctrl = e.ctrlKey && !e.altKey && !e.metaKey;
    // Ctrl+1 … Ctrl+3: the tabs; Ctrl+Tab and Ctrl+Shift+Tab go round them.
    if (ctrl && !e.shiftKey && /^Digit[1-3]$/.test(e.code)) {
      e.preventDefault();
      selectTab(tabs[Number(e.code.slice(5)) - 1].id);
      return;
    }
    if (ctrl && e.key === "Tab") {
      e.preventDefault();
      const at = tabs.findIndex((tab) => tab.id === view);
      selectTab(tabs[(at + (e.shiftKey ? tabs.length - 1 : 1)) % tabs.length].id);
      return;
    }
    // Ctrl+P: a break, pressed again it goes on. Ctrl+O: all resources of the current task.
    // Ctrl+L: into the quick entry, from wherever.
    if (ctrl && !e.shiftKey && e.code === "KeyP") {
      e.preventDefault();
      void run(snapshot?.on_break ? { kind: "resume" } : { kind: "pause" });
      return;
    }
    if (ctrl && !e.shiftKey && e.code === "KeyO") {
      e.preventDefault();
      if (focus[0]?.resources.length) void openFor(focus[0].id, null);
      else showToast({ text: t("The current task has no resources") });
      return;
    }
    if (ctrl && !e.shiftKey && e.code === "KeyL") {
      e.preventDefault();
      settings = false;
      elab = null;
      if (searching) closeSearch();
      else void tick().then(() => quick.focus());
      return;
    }
    // Ctrl+F opens the search, Esc closes it before it closes the popover (6.6).
    if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "f") {
      e.preventDefault();
      openSearch();
    } else if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "e") {
      e.preventDefault();
      if (elab) closeElab();
      else openElab(elabTarget());
    } else if (e.key === "Escape") {
      if (settings) closeSettings();
      else if (elab) closeElab();
      else if (searching) closeSearch();
      else void hidePopover();
    } else if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === "z" && !typing(e.target)) {
      e.preventDefault();
      void undoLast();
    }
  }
</script>

<svelte:window onkeydown={onKey} onpaste={onPaste} />

<div class="stage" class:preview>
  <div class="pop" class:wide>
    <Tabs
      {tabs}
      active={searching || settings ? "" : view}
      onselect={selectTab}
      onsearch={openSearch}
      {pinned}
      onpin={togglePin}
      onsettings={() => {
        searching = false;
        settings = true;
      }}
    />
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
      {#if searching}
        <SearchView
          version={snapshot}
          {run}
          open={(id, resource) => void openFor(id, resource)}
          onclose={closeSearch}
        />
      {:else if view === "idea"}
        <IdeasView {items} {run} open={(id, resource) => void openFor(id, resource)} onelaborate={openElab} />
      {:else if view === "delegated"}
        <DelegatedView {items} {run} open={(id, resource) => void openFor(id, resource)} onelaborate={openElab} />
      {:else if focus.length === 0}
        <div class="empty">
          <p><b>{t("Nothing urgent.")}</b><br />{t("Ctrl ⏎ in the quick entry puts a task into focus.")}</p>
        </div>
      {:else}
        <WorkCard
          item={focus[0]}
          onbreak={snapshot?.on_break ?? false}
          {run}
          {people}
          open={(resource) => void openFor(focus[0].id, resource)}
          openAll={() => void openFor(focus[0].id, null)}
          onelaborate={() => openElab(focus[0].id)}
        />
        <Queue
          bind:this={queue}
          items={focus.slice(1)}
          {run}
          open={(id, resource) => void openFor(id, resource)}
          onup={() => quick.focus()}
          onelaborate={openElab}
        />
        {#if focus.length >= MANY_IN_FOCUS}
          <p class="quiet">{t("Is all of this urgent?")}</p>
        {/if}
      {/if}
    </div>

    <IntermezzoBar {items} {run} />

    {#if elab}
      <div class="overlay elab">
        <ElaborateView
          {items}
          id={elab}
          {people}
          {run}
          open={(id, resource) => void openFor(id, resource)}
          onpick={(id) => (elab = id)}
          onclose={closeElab}
        />
      </div>
    {/if}

    {#if settings}
      <div class="overlay">
        <SettingsView onclose={closeSettings} />
      </div>
    {/if}

    <footer class="sx-chrome">
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
    background: var(--sx-bg);
    color: var(--sx-ink);
  }
  .preview .pop {
    width: 470px;
    height: 680px;
    border: 1px solid var(--sx-line);
    border-radius: var(--sx-radius-l);
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.38);
  }
  .preview .pop.wide {
    width: 940px;
  }
  .overlay.elab {
    z-index: 15;
    overflow: hidden;
  }
  .overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 20;
    background: var(--sx-bg);
    overflow-y: auto;
  }
  .body {
    flex: 1;
    overflow-x: hidden;
    overflow-y: auto;
    min-height: 0;
  }
  .quiet {
    margin: 12px 16px;
    font-size: 12.5px;
    color: var(--sx-ink-3);
    text-align: center;
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
  footer {
    padding: 10px 14px;
    font-size: 12px;
    border-top: 1px solid var(--sx-chrome-divider);
    color: var(--sx-ink-2);
  }
</style>
