<script lang="ts">
  import { onMount } from "svelte";
  import { hidePopover, inTauri, loadSnapshot, watchShown } from "./lib/backend";
  import type { Snapshot } from "./lib/types";

  // Plan step 0.2: a plain popover shell with data. The real views follow from step 1.5 on.
  let snapshot = $state<Snapshot | null>(null);
  const preview = !inTauri();

  const focus = $derived((snapshot?.items ?? []).filter((i) => i.state === "focus").sort((a, b) => a.order - b.order));
  const count = (state: string) => (snapshot?.items ?? []).filter((i) => i.state === state).length;

  onMount(() => {
    loadSnapshot().then((s) => (snapshot = s));
    const stop = watchShown(() => loadSnapshot().then((s) => (snapshot = s)));
    return () => void stop.then((f) => f());
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") void hidePopover();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="stage" class:preview>
  <div class="pop">
    <header>
      <span class="tab on">Fokus <b>{count("focus")}</b></span>
      <span class="tab">Ideen <b>{count("idea")}</b></span>
      <span class="tab">Delegiert <b>{count("delegated")}</b></span>
    </header>
    <ol>
      {#each focus as item (item.id)}
        <li class:active={item.active}>
          <span class="num">{item.order + 1}</span>
          <span class="title">{item.title}</span>
        </li>
      {:else}
        <li class="empty">Noch keine Einträge.</li>
      {/each}
    </ol>
    <footer>
      {#if preview}Browser-Vorschau mit Beispieldaten{:else}Esc versteckt das Popover{/if}
    </footer>
  </div>
</div>

<style>
  .stage {
    height: 100vh;
  }
  .stage.preview {
    display: grid;
    place-items: center;
    background: #2e3b47;
    height: 100vh;
  }
  .pop {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background: var(--tq-bg);
  }
  .preview .pop {
    width: 470px;
    height: 680px;
    border-radius: 14px;
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.38);
  }
  header {
    display: flex;
    gap: 4px;
    padding: 10px;
    border-bottom: 1px solid var(--tq-line);
  }
  .tab {
    padding: 7px 11px;
    border-radius: 8px;
    color: var(--tq-ink-2);
    font-weight: 500;
  }
  .tab.on {
    color: var(--tq-ink);
    background: var(--tq-card);
    box-shadow: 0 0 0 1px var(--tq-line);
  }
  .tab b {
    font-size: 12px;
    color: var(--tq-ink-2);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 1;
    overflow: auto;
  }
  li {
    display: flex;
    gap: 10px;
    padding: 11px 14px;
    border-bottom: 1px solid var(--tq-line);
  }
  li.active {
    font-weight: 600;
    color: var(--tq-accent);
  }
  .num {
    width: 18px;
    text-align: right;
    color: var(--tq-ink-2);
  }
  .empty {
    color: var(--tq-ink-2);
  }
  footer {
    padding: 10px 14px;
    font-size: 12px;
    color: var(--tq-ink-2);
    border-top: 1px solid var(--tq-line);
  }
</style>
