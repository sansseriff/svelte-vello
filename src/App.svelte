<script lang="ts">
  import { onMount } from "svelte";
  import { greet, main } from "../pkg/svelte_vello";
  import { add } from "../pkg/svelte_vello";
  // import { VelloContext } from "../pkg/svelte_vello";

  import { VelloContext } from "../pkg/svelte_vello";

  let vello: VelloContext | null = null;

  async function run() {
    const result = add(1, 2);
    console.log(`1 + 2 = ${result}`);
    if (result !== 3) throw new Error("wasm addition doesn't work!");
  }

  run();

  onMount(async () => {
    const cvs = document.getElementById("base_canvas") as HTMLCanvasElement;
    const dpr = window.devicePixelRatio || 1;

    // Set actual size in memory
    const rect = cvs.getBoundingClientRect();
    cvs.width = rect.width * dpr;
    cvs.height = rect.height * dpr;

    console.log("before creating context");
    let vello = await VelloContext.create("base_canvas");

    console.log("after creating context");

    // Add some shapes
    vello.add_rectangle(100, 100, 200, 150, 255, 0, 0); // Red rectangle
    vello.add_circle(400, 300, 50, 0, 0, 255); // Blue circle

    cvs.addEventListener("mousedown", (e) => {
      const rect = cvs.getBoundingClientRect();
      const x = (e.clientX - rect.left) * dpr;
      const y = (e.clientY - rect.top) * dpr;
      vello.handle_mouse_down(x, y);
    });

    cvs.addEventListener("mousemove", (e) => {
      const rect = cvs.getBoundingClientRect();
      const x = (e.clientX - rect.left) * dpr;
      const y = (e.clientY - rect.top) * dpr;
      vello.handle_mouse_move(x, y);
    });

    cvs.addEventListener("mouseup", () => {
      vello.handle_mouse_up();
    });
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
