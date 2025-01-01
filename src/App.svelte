<script lang="ts">
  import svelteLogo from "./assets/svelte.svg";
  import viteLogo from "/vite.svg";
  import Counter from "./lib/Counter.svelte";
  import { greet } from "../pkg/svelte_vello";
</script>

<main>
  <h2>{greet("The Friendly")}</h2>
</main>

<style>
</style>
