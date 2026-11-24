<script lang="ts">
  // Components page (plan 1.5, acceptance): every component in every theme available so far, light
  // and dark, to compare with the click dummy. Open with http://localhost:1430/#bausteine.
  import { ArrowUpToLine, Play } from "@lucide/svelte";
  import { t } from "./lib/i18n";
  import { mockSnapshot } from "./lib/mock-data";
  import { PRESETS, type Theme } from "./lib/theme";
  import Chip from "./lib/ui/Chip.svelte";
  import Notice from "./lib/ui/Notice.svelte";
  import Row from "./lib/ui/Row.svelte";
  import Section from "./lib/ui/Section.svelte";
  import Tabs from "./lib/ui/Tabs.svelte";
  import Toast from "./lib/ui/Toast.svelte";

  const items = mockSnapshot().items;
  const top = items.find((i) => i.active)!;
  const queue = items.filter((i) => i.state === "focus" && !i.active).sort((a, b) => a.order - b.order);
  const tabs = [
    { id: "focus", label: t("Focus"), count: 6 },
    { id: "idea", label: t("Ideas"), count: 4 },
    { id: "delegated", label: t("Delegated"), count: 2 },
  ];

  const byName = (name: string) => PRESETS.find((p) => p.name === name)!;
  const variants: { label: string; theme: Theme; scheme?: "light" | "dark" }[] = [
    { label: "Leitstand", theme: byName("Leitstand") },
    { label: "Automatisch · hell", theme: byName("Automatisch"), scheme: "light" },
    { label: "Automatisch · dunkel", theme: byName("Automatisch"), scheme: "dark" },
    { label: "Papier", theme: byName("Papier") },
    { label: "Graphit", theme: byName("Graphit") },
  ];
  let selected = $state(queue[1]?.id);
</script>

<main>
  <h1>Tasqlet · {t("Components")}</h1>
  <div class="grid">
    {#each variants as v (v.label)}
      <section
        class="frame"
        data-tq={v.theme.palette}
        data-focus={v.theme.focus}
        data-chrome={v.theme.chrome}
        style:color-scheme={v.scheme}
      >
        <h2>{t("Theme")}: {v.label}</h2>
        <div class="pop">
          <Tabs {tabs} active="focus" onselect={() => {}} onsearch={() => {}} />
          <Notice title={t("No tray found")} text="GNOME: AppIndicator and KStatusNotifierItem Support" />
          <Notice
            warn
            title={t("The data could not be loaded")}
            text={t("Tasqlet is read-only and changes nothing on disk.")}
            action={t("Restore the previous version")}
            onaction={() => {}}
          />
          <article class="card zone-focus">
            <h3>{top.title}</h3>
            <Section label={t("Resources")} onadd={() => {}}>
              <div class="chips">
                {#each top.resources.slice(0, 3) as r, i (r.id)}
                  <Chip resource={r} digit={i + 1} onopen={() => {}} onremove={() => {}} />
                {/each}
                <Chip onopen={() => {}} />
              </div>
            </Section>
            <Section label={t("Checklist")} onadd={() => {}}>
              <ul class="chk">
                {#each top.checklist as c (c.id)}
                  <li class:done={c.done}>
                    <label><input type="checkbox" checked={c.done} /> <span>{c.text}</span></label>
                  </li>
                {/each}
              </ul>
            </Section>
          </article>
          <div role="listbox" aria-label={t("Focus")}>
            {#each queue.slice(0, 3) as item, i (item.id)}
              <Row num={i + 2} title={item.title} selected={item.id === selected} onselect={() => (selected = item.id)}>
                {#snippet indicators()}
                  {#if i === 0}<span>{t("↩ last worked on")}</span>{/if}
                  {#if item.checklist.length}
                    <span>{item.checklist.filter((c) => c.done).length}/{item.checklist.length}</span>
                  {/if}
                {/snippet}
                {#snippet tools()}
                  <button class="ib" title={t("Up next (Alt+Home)")}><ArrowUpToLine size={16} /></button>
                  <button class="ib run" title={t("Take over now (Enter)")}><Play size={16} /></button>
                {/snippet}
              </Row>
            {/each}
          </div>
          <div class="toastwrap">
            <Toast message={{ text: t("Marked as done"), undo: () => {} }} />
          </div>
        </div>
      </section>
    {/each}
  </div>
</main>

<style>
  main {
    padding: 24px 28px 48px;
    background: #2e3b47;
    min-height: 100vh;
    color: #e6e9ec;
  }
  h1 {
    margin: 0 0 18px;
    font-size: 24px;
  }
  .grid {
    display: flex;
    flex-wrap: wrap;
    gap: 28px;
    align-items: flex-start;
  }
  .frame {
    width: 470px;
    background: none;
  }
  h2 {
    font-size: 13px;
    font-weight: 600;
    margin: 0 0 8px;
    color: #c9d1d9;
  }
  .pop {
    background: var(--tq-bg);
    color: var(--tq-ink);
    border: 1px solid var(--tq-line);
    border-radius: 14px;
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.38);
    padding-bottom: 12px;
  }
  .card {
    margin: 14px;
    padding: 14px 16px 16px;
    background: var(--tq-card);
    border: var(--tq-f-border-w) solid var(--tq-f-border);
    border-radius: 12px;
    box-shadow: var(--tq-f-shadow);
  }
  h3 {
    font-size: 17px;
    line-height: 1.3;
    margin: 0;
    font-weight: 600;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .chk {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .chk label {
    display: flex;
    gap: 9px;
    align-items: flex-start;
    padding: 2px 0;
  }
  .chk input {
    width: 16px;
    height: 16px;
    margin: 2px 0 0;
    accent-color: var(--tq-accent);
    flex: none;
  }
  .chk .done span {
    color: var(--tq-ink-2);
    text-decoration: line-through;
    text-decoration-color: var(--tq-ink-3);
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
  .toastwrap {
    margin: 12px 14px 0;
  }
</style>
