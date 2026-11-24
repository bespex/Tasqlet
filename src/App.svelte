<script lang="ts">
  import { schemaVersion } from "./lib/backend";

  // Plan step 0.1: an empty shell. The popover with mock data follows in step 0.2.
  const version = schemaVersion();
</script>

<main>
  <h1>Tasqlet</h1>
  {#await version then v}
    <p>Data format {v}</p>
  {/await}
</main>

<style>
  main {
    padding: 24px;
  }
  h1 {
    margin: 0 0 4px;
    font-size: 20px;
  }
  p {
    margin: 0;
    color: var(--tq-ink-2);
  }
</style>
