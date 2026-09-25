<script lang="ts">
  import { formatSeconds } from "../lib/api";
  import { drawChart, type ChartBar } from "../lib/chart";
  import type { DailyPoint } from "../lib/types";

  let { points }: { points: DailyPoint[] } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let bars = $state<ChartBar[]>([]);
  let width = $state(0);
  let tooltip = $state({ text: "", left: 0, top: 0 });

  $effect(() => {
    if (!canvas) {
      return;
    }
    const drawn = drawChart(canvas, points);
    bars = drawn.bars;
    width = drawn.width;
  });

  function hover(event: MouseEvent): void {
    if (!canvas) {
      return;
    }
    const bounds = canvas.getBoundingClientRect();
    const x = event.clientX - bounds.left;
    const hovered = bars.find((bar) => x >= bar.x - 2 && x <= bar.x + bar.width + 2);
    if (!hovered) {
      tooltip = { ...tooltip, text: "" };
      return;
    }
    tooltip = {
      text: `${hovered.point.day} · ${formatSeconds(hovered.point.seconds)} · ${hovered.point.sessions} session(s)`,
      left: Math.min(x + 12, Math.max(0, width - 190)),
      top: Math.max(6, hovered.y - 34),
    };
  }
</script>

<div class="chart-wrap">
  <canvas
    class="chart"
    bind:this={canvas}
    onmousemove={hover}
    onmouseleave={() => (tooltip = { ...tooltip, text: "" })}
  ></canvas>
  {#if tooltip.text}
    <div class="chart-tooltip" style="display:block;left:{tooltip.left}px;top:{tooltip.top}px">
      {tooltip.text}
    </div>
  {/if}
</div>
