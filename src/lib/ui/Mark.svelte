<script lang="ts">
  // Text with the search words marked (docs/KONZEPT.md 6.4, role `mark` in 7).
  let { text, terms }: { text: string; terms: string[] } = $props();

  const parts = $derived.by(() => {
    const words = terms.map((w) => w.replace(/^@/, "")).filter(Boolean);
    if (!words.length) return [{ text, hit: false }];
    const escaped = words.map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
    const found = text.split(new RegExp(`(${escaped.join("|")})`, "gi"));
    return found
      .filter((s) => s !== "")
      .map((s) => ({ text: s, hit: words.some((w) => w.toLowerCase() === s.toLowerCase()) }));
  });
</script>

{#each parts as p, i (i)}{#if p.hit}<mark>{p.text}</mark>{:else}{p.text}{/if}{/each}

<style>
  mark {
    background: var(--tq-mark);
    color: inherit;
    border-radius: 3px;
    padding: 0 1px;
  }
</style>
