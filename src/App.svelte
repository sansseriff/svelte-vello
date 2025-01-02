<script lang="ts">
  import { onMount } from "svelte";
  import { greet, main } from "../pkg/svelte_vello";
  import { add, setup_vello } from "../pkg/svelte_vello";

  async function run() {
    const result = add(1, 2);
    console.log(`1 + 2 = ${result}`);
    if (result !== 3) throw new Error("wasm addition doesn't work!");
  }

  run();

  onMount(() => {
    const canvas = document.getElementById("base_canvas") as HTMLCanvasElement;
    const dpr = window.devicePixelRatio || 1;

    // Set actual size in memory
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;

    setup_vello("base_canvas");
  });
</script>

<canvas id="base_canvas"></canvas>

<h2>{greet("The Friendly")}</h2>

<style>
  canvas {
    width: 50%;
    height: 50vh;
    display: block;
  }
</style>
