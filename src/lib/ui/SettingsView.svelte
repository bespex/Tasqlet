<script lang="ts">
  // Settings (plan 2.6) in tabs with a navigation on the left, as in Maillet: General (language,
  // popover position without an icon position, the global key, autostart), Theme (the kit's picker,
  // the same in every be spex app) and the learned Jira projects, forgotten one by one (D26).
  import "@bespex/themes/appearance";
  import type { Appearance } from "@bespex/themes";
  import { onDestroy, onMount } from "svelte";
  import {
    forgetJira,
    getSettings,
    setAppearance,
    setAutostart,
    setSettings,
    setToggleKey,
    suspendToggleKey,
  } from "../backend";
  import { currentLang, setLang, systemLang, t } from "../i18n";
  import { comboFromEvent, keyLabel } from "../keys";
  import { AXES_USED, appearanceOf, applyAppearance } from "../theme";
  import type { Language, PopoverPosition, Settings, ToggleKeyProblem } from "../types";

  // Without `onclose` this is the settings window (D40): the window's own frame closes it.
  let { onclose }: { onclose?: () => void } = $props();

  let s = $state<Settings | null>(null);

  type Section = "general" | "theme" | "jira";
  let section = $state<Section>("general");
  const sections: { id: Section; label: string }[] = $derived([
    { id: "general", label: t("General") },
    { id: "theme", label: t("Theme") },
    { id: "jira", label: t("Jira projects") },
  ]);
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

  // The global key (D36): click the field, press the combination; Esc cancels, Backspace clears.
  let recording = $state(false);
  let keyProblem = $state("");
  const problems = (code: ToggleKeyProblem | string): string =>
    code === "taken"
      ? t("This key is already taken, by another program or the desktop. The previous key stays.")
      : code === "invalid"
        ? t("This key cannot be used.")
        : code === "unsupported"
          ? t("Wayland does not let apps hold global keys.")
          : code;
  async function record(on: boolean) {
    if (recording === on) return;
    recording = on;
    await suspendToggleKey(on);
  }
  function onRecordKey(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    const plain = !e.ctrlKey && !e.altKey && !e.metaKey && !e.shiftKey;
    if (plain && e.key === "Escape") return void record(false);
    if (plain && (e.key === "Backspace" || e.key === "Delete")) return void keepKey("");
    const combo = comboFromEvent(e);
    if (combo) void keepKey(combo);
  }
  async function keepKey(combo: string) {
    keyProblem = "";
    try {
      await setToggleKey(combo);
    } catch (e) {
      keyProblem = problems(String(e));
    }
    await record(false);
    await load();
  }
  onDestroy(() => {
    if (recording) void suspendToggleKey(false);
  });

  // A choice shows at once and is kept; the picker only offers values the kit knows.
  const choose = (value: Appearance) => {
    applyAppearance(value);
    void guard(() => setAppearance(value));
  };

  const save = (language: Language, position: PopoverPosition) =>
    guard(async () => {
      await setSettings(language, position);
      setLang(language === "auto" ? systemLang() : language);
    });
</script>

<div class="settings" class:settings-window={!onclose}>
  <header>
    <h2>{t("Settings")}</h2>
    {#if onclose}<button class="back" onclick={onclose} title="Esc">{t("Close")}</button>{/if}
  </header>
  <div class="cols">
    <nav aria-label={t("Settings")}>
      {#each sections as sec (sec.id)}
        <button
          class:on={section === sec.id}
          aria-current={section === sec.id ? "page" : undefined}
          onclick={() => (section = sec.id)}
        >
          {sec.label}
        </button>
      {/each}
    </nav>
    <div class="content">
      {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
      {#if s && section === "general"}
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

        <div class="row">
          <span>{t("Key to show or hide")}</span>
          {#if s.wayland}
            <span class="hint">{t("Not on Wayland")}</span>
          {:else}
            <span class="keys">
              <button
                class="record"
                class:on={recording}
                onclick={() => void record(!recording)}
                onkeydown={onRecordKey}
                onblur={() => void record(false)}
              >
                {recording ? t("Press keys …") : s.toggle_key ? keyLabel(s.toggle_key) : t("None")}
              </button>
              {#if s.toggle_key && !recording}
                <button title={t("Remove")} aria-label={t("Remove")} onclick={() => void keepKey("")}>×</button>
              {/if}
            </span>
          {/if}
        </div>
        {#if s.wayland}
          <p class="hint">{t("Put “tasqlet --toggle” on a key in your desktop's keyboard settings.")}</p>
        {:else if recording}
          <p class="hint">{t("Esc cancels, Backspace removes the key.")}</p>
        {:else if keyProblem}
          <p class="hint warn" role="alert">{keyProblem}</p>
        {:else if s.toggle_key_problem}
          <p class="hint warn">{problems(s.toggle_key_problem)}</p>
        {:else}
          <p class="hint">{t("Works everywhere, also when Tasqlet is hidden.")}</p>
        {/if}

        <label class="row check">
          <input
            type="checkbox"
            checked={s.autostart}
            onchange={(e) => guard(() => setAutostart(e.currentTarget.checked))}
          />
          <span>{t("Start Tasqlet at login")}</span>
        </label>
      {:else if s && section === "theme"}
        <p class="hint lead">{t("The same choice as in every be spex app. A click applies it at once.")}</p>
        <sx-appearance
          class="appearance"
          lang={currentLang()}
          axes={AXES_USED.join(" ")}
          value={appearanceOf(s.appearance)}
          onchange={(e: CustomEvent<Appearance>) => choose(e.detail)}
        ></sx-appearance>
      {:else if s && section === "jira"}
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
  </div>
</div>

<style>
  .settings {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  header {
    flex: none;
    padding: 14px 16px 10px;
    border-bottom: 1px solid var(--sx-chrome-divider, var(--sx-line));
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  h2 {
    margin: 0;
    font-size: 15px;
  }
  .cols {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 160px minmax(0, 1fr);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 8px;
    border-right: 1px solid var(--sx-line);
    background: var(--sx-panel);
  }
  nav button {
    text-align: left;
    border: 0;
    background: none;
    padding: 6px 10px;
    border-radius: var(--sx-radius-m);
    color: var(--sx-ink-2);
  }
  nav button:hover {
    background: var(--sx-hover);
    color: var(--sx-ink);
  }
  nav button.on {
    background: var(--sx-accent-soft);
    color: var(--sx-accent-ink);
    font-weight: 600;
  }
  .content {
    overflow-y: auto;
    padding: 4px 16px 16px;
  }
  .lead {
    margin-top: 12px;
  }
  sx-appearance::part(label) {
    margin: 14px 0 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--sx-ink-2);
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
    color: var(--sx-ink);
    background: var(--sx-field);
    border: 1px solid var(--sx-line-strong);
    border-radius: var(--sx-radius-l);
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
    border-radius: var(--sx-radius-l) 0 0 var(--sx-radius-l);
    margin-left: 0;
  }
  .seg button:last-child {
    border-radius: 0 var(--sx-radius-l) var(--sx-radius-l) 0;
  }
  .seg .on {
    background: var(--sx-accent-soft);
    color: var(--sx-accent-ink);
    border-color: var(--sx-accent);
    position: relative;
    z-index: 1;
    font-weight: 600;
  }
  button:hover {
    border-color: var(--sx-accent);
  }
  .hint {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--sx-ink-3);
  }
  .keys {
    display: inline-flex;
    gap: 6px;
  }
  .record {
    min-width: 120px;
  }
  .record.on {
    border-color: var(--sx-accent);
    color: var(--sx-accent-ink);
    background: var(--sx-accent-soft);
  }
  .warn {
    color: var(--sx-warn, var(--sx-ink));
  }
  .problem {
    margin: 8px 0 0;
    font-size: 12.5px;
    color: var(--sx-warn, var(--sx-ink));
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
    color: var(--sx-ink-3);
  }
</style>
