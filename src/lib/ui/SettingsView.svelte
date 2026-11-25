<script lang="ts">
  // Slim settings (plan 2.6 in a slim form): language, popover position without an icon position,
  // autostart and the learned Jira projects, which can be forgotten one by one (D26). Themes follow
  // with the rest of 2.6.
  import { onMount } from "svelte";
  import { forgetJira, getSettings, setAutostart, setSettings } from "../backend";
  import { setLang, systemLang, t } from "../i18n";
  import type { Language, PopoverPosition, Settings } from "../types";

  let { onclose }: { onclose: () => void } = $props();

  let s = $state<Settings | null>(null);
  let problem = $state("");

  const load = () => getSettings().then((v) => (s = v));
  onMount(() => void load());

  async function guard(job: () => Promise<void>) {
    problem = "";
    try {
      await job();
    } catch (e) {
      problem = String(e);
    }
    await load();
  }

  const langs: { value: Language; label: string }[] = $derived([
    { value: "auto", label: t("System") },
    { value: "de", label: "Deutsch" },
    { value: "en", label: "English" },
  ]);
  const positions: { value: PopoverPosition; label: string }[] = $derived([
    { value: "corner", label: t("Corner") },
    { value: "center", label: t("Middle") },
  ]);

  const save = (language: Language, position: PopoverPosition) =>
    guard(async () => {
      await setSettings(language, position);
      setLang(language === "auto" ? systemLang() : language);
    });
</script>

<div class="settings">
  <header>
    <h2>{t("Settings")}</h2>
    <button class="back" onclick={onclose} title="Esc">{t("Close")}</button>
  </header>
  {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
  {#if s}
    <!-- No native <select>: its drop-down is a window of its own, which takes the focus from the
         popover and so closes it. -->
    <div class="row" role="group" aria-label={t("Language")}>
      <span>{t("Language")}</span>
      <div class="seg">
        {#each langs as o (o.value)}
          <button
            class:on={s.language === o.value}
            aria-pressed={s.language === o.value}
            onclick={() => save(o.value, s!.popover_position)}
          >
            {o.label}
          </button>
        {/each}
      </div>
    </div>
    <p class="hint">{t("The tray texts change at the next start.")}</p>

    <div class="row" role="group" aria-label={t("Popover without icon position")}>
      <span>{t("Popover without icon position")}</span>
      <div class="seg">
        {#each positions as o (o.value)}
          <button
            class:on={s.popover_position === o.value}
            aria-pressed={s.popover_position === o.value}
            onclick={() => save(s!.language, o.value)}
          >
            {o.label}
          </button>
        {/each}
      </div>
    </div>
    {#if !s.wayland}<p class="hint">
        {t("On X11 the popover sits at the tray icon; this applies to the key binding.")}
      </p>{/if}

    <label class="row check">
      <input
        type="checkbox"
        checked={s.autostart}
        onchange={(e) => guard(() => setAutostart(e.currentTarget.checked))}
      />
      <span>{t("Start Tasqlet at login")}</span>
    </label>

    <h3>{t("Jira projects")}</h3>
    {#if s.jira.length === 0}
      <p class="hint">{t("Nothing learned yet. A Jira link in an entry teaches its project.")}</p>
    {:else}
      <ul role="list">
        {#each s.jira as j (j.project)}
          <li>
            <b>{j.project}</b><span class="base">{j.base}</span>
            <button onclick={() => guard(() => forgetJira(j.project))}>{t("Forget")}</button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .settings {
    padding: 12px 16px 16px;
  }
  header {
    padding: 4px 0 10px;
    margin: 0 -16px;
    padding-inline: 16px;
    border-bottom: 1px solid var(--tq-c-divider, var(--tq-line));
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  h2 {
    margin: 0;
    font-size: 15px;
  }
  h3 {
    margin: 18px 0 6px;
    font-size: 13px;
    color: var(--tq-ink-2);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 12px;
    font-size: 13.5px;
  }
  .check {
    justify-content: flex-start;
  }
  button {
    font: inherit;
    font-size: 13px;
    color: var(--tq-ink);
    background: var(--tq-field);
    border: 1px solid var(--tq-line-strong);
    border-radius: 8px;
    padding: 4px 10px;
  }
  .seg {
    display: inline-flex;
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
  button:hover {
    border-color: var(--tq-accent);
  }
  .hint {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--tq-ink-3);
  }
  .problem {
    margin: 8px 0 0;
    font-size: 12.5px;
    color: var(--tq-warn, var(--tq-ink));
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 6px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }
  .base {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--tq-ink-3);
  }
</style>
