<script lang="ts">
  // The feedback line (docs/KONZEPT.md 7.1): inverted (ink as surface), so it suits every theme.
  import { Undo2 } from "@lucide/svelte";
  import { t } from "../i18n";
  import { hideToast, toast, type ToastMessage } from "../toast.svelte";

  /** A fixed message instead of the shared one (components page). */
  let { message }: { message?: ToastMessage } = $props();
  const shown = $derived(message ?? toast.current);

  function undo() {
    const run = shown?.undo;
    if (!message) hideToast();
    run?.();
  }
</script>

{#if shown}
  <div class="toast" class:fixed={!!message} role="status">
    <span>{shown.text}</span>
    {#if shown.undo}
      <button onclick={undo}><Undo2 size={14} />{t("Undo")}</button>
    {/if}
  </div>
{/if}

<style>
  .toast {
    position: absolute;
    left: 14px;
    right: 14px;
    bottom: 60px;
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--tq-ink);
    color: var(--tq-bg);
    border-radius: 10px;
    padding: 9px 10px 9px 14px;
    font-size: 13px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
    z-index: 7;
    color-scheme: var(--tq-scheme-inv);
  }
  .toast.fixed {
    position: static;
  }
  span {
    flex: 1;
  }
  button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: color-mix(in srgb, var(--tq-bg) 16%, transparent);
    border: 0;
    color: var(--tq-bg);
    border-radius: 7px;
    padding: 5px 9px;
    font-weight: 600;
    white-space: nowrap;
  }
</style>
